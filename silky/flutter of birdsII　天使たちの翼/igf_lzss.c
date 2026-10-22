/*
 * igf_lzss.c — Fast LZSS decompression/compression for Silky engine (Angel.exe / FOB2).
 *
 * Decompressor: exact match for Angel_chs.exe's LZSS variant.
 * Compressor:   matches the Python reference implementation (ifl_tool_engine.py),
 *               using 24-bit hash keys, proper overlapping-match overlay, and
 *               distance-preferring candidate ordering.
 *
 * Build (MinGW x64):
 *   gcc -shared -O3 -o igf_lzss.dll igf_lzss.c
 */

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#ifdef _MSC_VER
#define DLL_EXPORT __declspec(dllexport)
#else
#define DLL_EXPORT __attribute__((visibility("default")))
#endif

#define FRAME_SIZE  0x1000
#define FRAME_INIT  0xFEE
#define FRAME_FILL  0x20
#define FRAME_MASK  0xFFF

#define HASH_BITS   16
#define HASH_SIZE   (1 << HASH_BITS)
#define HASH_MASK   (HASH_SIZE - 1)

/* ─── Decompress ─────────────────────────────────────────────────────── */

DLL_EXPORT int lzss_decompress(
    const uint8_t *src, size_t src_len,
    size_t expected_size,
    uint8_t **out, size_t *out_len)
{
    uint8_t *dst;
    uint8_t  frame[FRAME_SIZE];
    size_t   frame_pos;
    int      flags;
    size_t   i;

    if (!src || !out || !out_len) return -1;

    dst = (uint8_t *)malloc(expected_size ? expected_size : src_len * 4);
    if (!dst) return -2;

    memset(frame, FRAME_FILL, FRAME_SIZE);
    frame_pos = FRAME_INIT;
    flags = 0;
    i = 0;
    *out_len = 0;

    while (i < src_len) {
        flags >>= 1;
        if ((flags & 0x100) == 0) {
            if (i >= src_len) break;
            flags = src[i++] | 0xFF00;
        }
        if (flags & 1) {
            if (i >= src_len) break;
            uint8_t b = src[i++];
            if (*out_len < expected_size || !expected_size)
                dst[(*out_len)++] = b;
            frame[frame_pos] = b;
            frame_pos = (frame_pos + 1) & FRAME_MASK;
        } else {
            if (i + 1 >= src_len) break;
            uint8_t  lo = src[i++];
            uint8_t  hi = src[i++];
            uint16_t pos = lo | ((uint16_t)(hi & 0xF0) << 4);
            int      len = (hi & 0x0F) + 3;

            for (int k = 0; k < len; k++) {
                uint8_t b = frame[(pos + k) & FRAME_MASK];
                if (*out_len < expected_size || !expected_size)
                    dst[(*out_len)++] = b;
                frame[frame_pos] = b;
                frame_pos = (frame_pos + 1) & FRAME_MASK;
                if (expected_size && *out_len >= expected_size)
                    goto done;
            }
        }
    }
done:
    *out = dst;
    return 0;
}

/* ─── 24-bit-key hash index for compression ──────────────────────────── */

/* Multiplicative hash: full 24-bit key → 16-bit bucket */
static inline uint32_t hash24(uint32_t key24) {
    return (key24 * 2654435761u) >> (32 - HASH_BITS);
}

typedef struct {
    uint8_t  *frame;          /* pointer to frame ring buffer */
    uint32_t  pos_key24[FRAME_SIZE]; /* full 24-bit key per position */
    int       bucket_heads[HASH_SIZE];
    int       bucket_next[FRAME_SIZE];
} LzssIndex;

static inline uint32_t frame_key24(const uint8_t *frame, int pos) {
    return ((uint32_t)frame[pos & FRAME_MASK])
         | ((uint32_t)frame[(pos + 1) & FRAME_MASK] << 8)
         | ((uint32_t)frame[(pos + 2) & FRAME_MASK] << 16);
}

static void index_init(LzssIndex *idx, uint8_t *frame) {
    idx->frame = frame;
    memset(idx->bucket_heads, -1, sizeof(idx->bucket_heads));
    for (int pos = 0; pos < FRAME_SIZE; pos++) {
        uint32_t k24 = frame_key24(frame, pos);
        uint32_t h = hash24(k24);
        idx->pos_key24[pos] = k24;
        idx->bucket_next[pos] = idx->bucket_heads[h];
        idx->bucket_heads[h] = pos;
    }
}

static void index_put_byte(LzssIndex *idx, int frame_pos, uint8_t value) {
    int fp = frame_pos & FRAME_MASK;
    idx->frame[fp] = value;

    /* Update keys for three positions whose 3-byte window includes fp */
    for (int off = -2; off <= 0; off++) {
        int p = (fp + off) & FRAME_MASK;
        uint32_t new_k24 = frame_key24(idx->frame, p);
        if (new_k24 == idx->pos_key24[p]) continue;

        /* Remove from old bucket */
        uint32_t old_k24 = idx->pos_key24[p];
        uint32_t old_h = hash24(old_k24);
        int *head = &idx->bucket_heads[old_h];
        int cur = *head, prev = -1;
        while (cur >= 0 && cur != p) {
            prev = cur;
            cur = idx->bucket_next[cur];
        }
        if (cur == p) {
            if (prev >= 0)
                idx->bucket_next[prev] = idx->bucket_next[p];
            else
                *head = idx->bucket_next[p];
        }

        /* Add to new bucket */
        idx->pos_key24[p] = new_k24;
        uint32_t new_h = hash24(new_k24);
        idx->bucket_next[p] = idx->bucket_heads[new_h];
        idx->bucket_heads[new_h] = p;
    }
}

/*
 * Find longest match with correct self-overlap handling.
 *
 * When a candidate match overlaps the current write position and the match
 * is longer than 3 bytes, the decompressor will overwrite frame bytes that
 * are later used as the match source.  We must simulate this overlay so
 * that the compressor only emits matches that the decompressor can faithfully
 * reproduce.
 */
static void index_find_match(
    LzssIndex *idx, int frame_pos,
    const uint8_t *look, int look_len, int max_len,
    int *out_pos, int *out_len)
{
    int best_pos = 0, best_len = 0;
    int limit = (max_len < look_len) ? max_len : look_len;
    if (limit < 3) { *out_pos = 0; *out_len = 0; return; }

    uint32_t look_k24 = look[0] | ((uint32_t)look[1] << 8) | ((uint32_t)look[2] << 16);
    uint32_t h = hash24(look_k24);
    int fp = frame_pos & FRAME_MASK;

    uint8_t *frame = idx->frame;

    for (int cand = idx->bucket_heads[h]; cand >= 0; cand = idx->bucket_next[cand]) {
        if (idx->pos_key24[cand] != look_k24) continue;

        /* Simulate decompressor's frame writes for this candidate match.
         * Build a small overlay to model bytes that would be overwritten
         * during the match expansion by the decompressor. */
        uint8_t overlay[18];      /* at most 18 bytes can be overwritten */
        int     overlay_pos[18];  /* frame positions of overlay entries */
        int     overlay_count = 0;
        int     sim_fp = fp;      /* simulated write head */
        int     len = 0;

        while (len < limit) {
            int rp = (cand + len) & FRAME_MASK;

            /* Check overlay first: has this frame position been
             * overwritten by the simulated match expansion? */
            uint8_t b;
            int found = 0;
            for (int oi = 0; oi < overlay_count; oi++) {
                if (overlay_pos[oi] == rp) {
                    b = overlay[oi];
                    found = 1;
                    break;
                }
            }
            if (!found)
                b = frame[rp];

            if (b != look[len]) break;

            /* Simulate writing this byte to the frame */
            overlay[overlay_count] = look[len];
            overlay_pos[overlay_count] = sim_fp;
            overlay_count++;
            if (overlay_count > 18) overlay_count = 18; /* safety clamp */

            len++;
            sim_fp = (sim_fp + 1) & FRAME_MASK;
        }

        if (len > best_len) {
            best_len = len;
            best_pos = cand;
            if (best_len >= limit) break;
        }
    }

    if (best_len < 3) { *out_pos = 0; *out_len = 0; return; }
    *out_pos = best_pos;
    *out_len = best_len;
}

/* ─── Compress ───────────────────────────────────────────────────────── */

DLL_EXPORT int lzss_compress(
    const uint8_t *src, size_t src_len,
    int level,
    uint8_t **out, size_t *out_len)
{
    uint8_t   frame[FRAME_SIZE];
    LzssIndex idx;
    size_t    si, di;
    uint8_t  *dst;
    size_t    dst_cap;
    int       candidate_cap;

    if (!src || !out || !out_len) return -1;

    switch (level) {
        case 0: candidate_cap = 32;  break;
        case 2: candidate_cap = 4096; break;
        default: candidate_cap = 384; break;
    }

    memset(frame, FRAME_FILL, FRAME_SIZE);
    index_init(&idx, frame);

    dst_cap = src_len + src_len / 8 + 256;
    dst = (uint8_t *)malloc(dst_cap);
    if (!dst) return -2;

    si = 0;
    di = 0;
    int frame_pos = FRAME_INIT;

    while (si < src_len) {
        size_t flag_pos = di++;
        if (di >= dst_cap) { dst_cap *= 2; dst = (uint8_t *)realloc(dst, dst_cap); }
        dst[flag_pos] = 0;
        int flags = 0;

        for (int bit = 0; bit < 8; bit++) {
            if (si >= src_len) break;

            int match_pos, match_len;
            int remain = (int)(src_len - si);
            int max_look = (remain < 18) ? remain : 18;

            index_find_match(&idx, frame_pos, src + si, max_look, 18,
                           &match_pos, &match_len);

            if (match_len >= 3) {
                /* output match */
                if (di + 2 >= dst_cap) { dst_cap *= 2; dst = (uint8_t *)realloc(dst, dst_cap); }
                dst[di++] = match_pos & 0xFF;
                dst[di++] = ((match_pos >> 4) & 0xF0) | ((match_len - 3) & 0x0F);

                for (int k = 0; k < match_len; k++) {
                    index_put_byte(&idx, frame_pos, src[si + k]);
                    frame_pos = (frame_pos + 1) & FRAME_MASK;
                }
                si += match_len;
            } else {
                /* output literal */
                if (di >= dst_cap) { dst_cap *= 2; dst = (uint8_t *)realloc(dst, dst_cap); }
                uint8_t b = src[si++];
                flags |= (1 << bit);
                dst[di++] = b;
                index_put_byte(&idx, frame_pos, b);
                frame_pos = (frame_pos + 1) & FRAME_MASK;
            }
        }
        dst[flag_pos] = (uint8_t)flags;
    }

    *out = dst;
    *out_len = di;
    return 0;
}

/* ─── Free buffer ────────────────────────────────────────────────────── */

DLL_EXPORT void lzss_free(void *ptr) {
    free(ptr);
}

/* ─── Pixel conversion ───────────────────────────────────────────────── */

DLL_EXPORT void bgr_to_rgb(
    const uint8_t *src, uint8_t *dst,
    int width, int height, int src_stride)
{
    if (src_stride <= 0) src_stride = (width * 3 + 3) & ~3;
    int dst_stride = width * 3;

    for (int y = 0; y < height; y++) {
        const uint8_t *srow = src + (height - 1 - y) * src_stride;
        uint8_t       *drow = dst + y * dst_stride;
        for (int x = 0; x < width; x++) {
            drow[x * 3 + 0] = srow[x * 3 + 2];
            drow[x * 3 + 1] = srow[x * 3 + 1];
            drow[x * 3 + 2] = srow[x * 3 + 0];
        }
    }
}

DLL_EXPORT void rgb_to_bgr(
    const uint8_t *src, uint8_t *dst,
    int width, int height, int dst_stride)
{
    if (dst_stride <= 0) dst_stride = (width * 3 + 3) & ~3;

    for (int y = 0; y < height; y++) {
        const uint8_t *srow = src + y * width * 3;
        uint8_t       *drow = dst + (height - 1 - y) * dst_stride;
        for (int x = 0; x < width; x++) {
            drow[x * 3 + 0] = srow[x * 3 + 2];
            drow[x * 3 + 1] = srow[x * 3 + 1];
            drow[x * 3 + 2] = srow[x * 3 + 0];
        }
        for (int p = width * 3; p < dst_stride; p++) {
            drow[p] = 0;
        }
    }
}
