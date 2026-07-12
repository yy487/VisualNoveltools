#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define FRAME_SIZE 0x1000
#define FRAME_INIT 0xFEE
#define FRAME_FILL 0x00
#define FRAME_MASK 0xFFF
#define MIN_MATCH 3
#define MAX_MATCH 18

#define HASH_BITS 16
#define HASH_SIZE (1 << HASH_BITS)

typedef struct {
    uint8_t *frame;
    uint32_t pos_key24[FRAME_SIZE];
    int bucket_heads[HASH_SIZE];
    int bucket_next[FRAME_SIZE];
} LzssIndex;

static uint32_t hash24(uint32_t key24) {
    return (key24 * 2654435761u) >> (32 - HASH_BITS);
}

static uint32_t frame_key24(const uint8_t *frame, int pos) {
    return ((uint32_t)frame[pos & FRAME_MASK])
         | ((uint32_t)frame[(pos + 1) & FRAME_MASK] << 8)
         | ((uint32_t)frame[(pos + 2) & FRAME_MASK] << 16);
}

static void index_init(LzssIndex *idx, uint8_t *frame) {
    idx->frame = frame;
    for (int i = 0; i < HASH_SIZE; i++) {
        idx->bucket_heads[i] = -1;
    }
    for (int pos = 0; pos < FRAME_SIZE; pos++) {
        uint32_t key = frame_key24(frame, pos);
        uint32_t h = hash24(key);
        idx->pos_key24[pos] = key;
        idx->bucket_next[pos] = idx->bucket_heads[h];
        idx->bucket_heads[h] = pos;
    }
}

static void index_put_byte(LzssIndex *idx, int frame_pos, uint8_t value) {
    int fp = frame_pos & FRAME_MASK;
    idx->frame[fp] = value;

    for (int off = -2; off <= 0; off++) {
        int p = (fp + off) & FRAME_MASK;
        uint32_t new_key = frame_key24(idx->frame, p);
        if (new_key == idx->pos_key24[p]) {
            continue;
        }

        uint32_t old_key = idx->pos_key24[p];
        uint32_t old_h = hash24(old_key);
        int *head = &idx->bucket_heads[old_h];
        int cur = *head;
        int prev = -1;
        while (cur >= 0 && cur != p) {
            prev = cur;
            cur = idx->bucket_next[cur];
        }
        if (cur == p) {
            if (prev >= 0) {
                idx->bucket_next[prev] = idx->bucket_next[p];
            } else {
                *head = idx->bucket_next[p];
            }
        }

        idx->pos_key24[p] = new_key;
        uint32_t new_h = hash24(new_key);
        idx->bucket_next[p] = idx->bucket_heads[new_h];
        idx->bucket_heads[new_h] = p;
    }
}

static void index_find_match(
    LzssIndex *idx,
    int frame_pos,
    const uint8_t *look,
    int look_len,
    int *out_pos,
    int *out_len)
{
    int limit = look_len < MAX_MATCH ? look_len : MAX_MATCH;
    if (limit < MIN_MATCH) {
        *out_pos = 0;
        *out_len = 0;
        return;
    }

    uint32_t look_key = look[0] | ((uint32_t)look[1] << 8) | ((uint32_t)look[2] << 16);
    uint32_t h = hash24(look_key);
    uint8_t *frame = idx->frame;
    int best_pos = 0;
    int best_len = 0;
    int fp = frame_pos & FRAME_MASK;

    for (int cand = idx->bucket_heads[h]; cand >= 0; cand = idx->bucket_next[cand]) {
        if (idx->pos_key24[cand] != look_key) {
            continue;
        }

        uint8_t overlay[MAX_MATCH];
        int overlay_pos[MAX_MATCH];
        int overlay_count = 0;
        int sim_fp = fp;
        int len = 0;

        while (len < limit) {
            int rp = (cand + len) & FRAME_MASK;
            uint8_t b = 0;
            int found = 0;
            for (int i = 0; i < overlay_count; i++) {
                if (overlay_pos[i] == rp) {
                    b = overlay[i];
                    found = 1;
                    break;
                }
            }
            if (!found) {
                b = frame[rp];
            }
            if (b != look[len]) {
                break;
            }

            if (overlay_count < MAX_MATCH) {
                overlay[overlay_count] = look[len];
                overlay_pos[overlay_count] = sim_fp;
                overlay_count++;
            }
            sim_fp = (sim_fp + 1) & FRAME_MASK;
            len++;
        }

        if (len > best_len) {
            best_len = len;
            best_pos = cand;
            if (best_len == limit) {
                break;
            }
        }
    }

    if (best_len < MIN_MATCH) {
        *out_pos = 0;
        *out_len = 0;
        return;
    }
    *out_pos = best_pos;
    *out_len = best_len;
}

static int compress_lzss(const uint8_t *src, size_t src_len, uint8_t **out, size_t *out_len) {
    uint8_t frame[FRAME_SIZE];
    memset(frame, FRAME_FILL, sizeof(frame));

    LzssIndex idx;
    index_init(&idx, frame);

    size_t dst_cap = src_len + src_len / 8 + 256;
    uint8_t *dst = (uint8_t *)malloc(dst_cap);
    if (!dst) {
        return -1;
    }

    size_t si = 0;
    size_t di = 0;
    int frame_pos = FRAME_INIT;

    while (si < src_len) {
        size_t flag_pos = di++;
        if (di >= dst_cap) {
            dst_cap *= 2;
            uint8_t *tmp = (uint8_t *)realloc(dst, dst_cap);
            if (!tmp) {
                free(dst);
                return -2;
            }
            dst = tmp;
        }
        dst[flag_pos] = 0;
        int flags = 0;

        for (int bit = 0; bit < 8; bit++) {
            if (si >= src_len) {
                break;
            }

            int match_pos = 0;
            int match_len = 0;
            int remain = (int)(src_len - si);
            index_find_match(&idx, frame_pos, src + si, remain, &match_pos, &match_len);

            if (match_len >= MIN_MATCH) {
                if (di + 2 >= dst_cap) {
                    dst_cap *= 2;
                    uint8_t *tmp = (uint8_t *)realloc(dst, dst_cap);
                    if (!tmp) {
                        free(dst);
                        return -2;
                    }
                    dst = tmp;
                }
                dst[di++] = (uint8_t)(match_pos & 0xFF);
                dst[di++] = (uint8_t)(((match_pos >> 4) & 0xF0) | ((match_len - MIN_MATCH) & 0x0F));

                for (int k = 0; k < match_len; k++) {
                    index_put_byte(&idx, frame_pos, src[si + k]);
                    frame_pos = (frame_pos + 1) & FRAME_MASK;
                }
                si += (size_t)match_len;
            } else {
                if (di >= dst_cap) {
                    dst_cap *= 2;
                    uint8_t *tmp = (uint8_t *)realloc(dst, dst_cap);
                    if (!tmp) {
                        free(dst);
                        return -2;
                    }
                    dst = tmp;
                }
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

static int decompress_lzss(const uint8_t *src, size_t src_len, size_t expected, uint8_t **out, size_t *out_len) {
    uint8_t *dst = (uint8_t *)malloc(expected ? expected : src_len * 8 + 4096);
    if (!dst) {
        return -1;
    }

    uint8_t frame[FRAME_SIZE];
    memset(frame, FRAME_FILL, sizeof(frame));

    int frame_pos = FRAME_INIT;
    int flags = 0;
    size_t si = 0;
    size_t di = 0;
    size_t cap = expected ? expected : src_len * 8 + 4096;

    while (si < src_len) {
        flags >>= 1;
        if ((flags & 0x100) == 0) {
            flags = src[si++] | 0xFF00;
        }

        if (flags & 1) {
            if (si >= src_len) {
                break;
            }
            uint8_t b = src[si++];
            if (di >= cap) {
                free(dst);
                return -2;
            }
            dst[di++] = b;
            frame[frame_pos] = b;
            frame_pos = (frame_pos + 1) & FRAME_MASK;
        } else {
            if (si + 1 >= src_len) {
                break;
            }
            uint8_t lo = src[si++];
            uint8_t hi = src[si++];
            int pos = lo | ((hi & 0xF0) << 4);
            int len = (hi & 0x0F) + MIN_MATCH;
            for (int k = 0; k < len; k++) {
                uint8_t b = frame[(pos + k) & FRAME_MASK];
                if (di >= cap) {
                    free(dst);
                    return -2;
                }
                dst[di++] = b;
                frame[frame_pos] = b;
                frame_pos = (frame_pos + 1) & FRAME_MASK;
                if (expected && di >= expected) {
                    *out = dst;
                    *out_len = di;
                    return 0;
                }
            }
        }
    }

    *out = dst;
    *out_len = di;
    return 0;
}

static int read_file(const char *path, uint8_t **buf, size_t *len) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        return -1;
    }
    if (fseek(f, 0, SEEK_END) != 0) {
        fclose(f);
        return -2;
    }
    long n = ftell(f);
    if (n < 0) {
        fclose(f);
        return -3;
    }
    if (fseek(f, 0, SEEK_SET) != 0) {
        fclose(f);
        return -4;
    }
    uint8_t *data = (uint8_t *)malloc((size_t)n);
    if (!data) {
        fclose(f);
        return -5;
    }
    if (fread(data, 1, (size_t)n, f) != (size_t)n) {
        free(data);
        fclose(f);
        return -6;
    }
    fclose(f);
    *buf = data;
    *len = (size_t)n;
    return 0;
}

static int write_file(const char *path, const uint8_t *buf, size_t len) {
    FILE *f = fopen(path, "wb");
    if (!f) {
        return -1;
    }
    int ok = fwrite(buf, 1, len, f) == len;
    fclose(f);
    return ok ? 0 : -2;
}

int main(int argc, char **argv) {
    if (argc != 3) {
        fprintf(stderr, "usage: %s input.raw output.mr\n", argv[0]);
        return 2;
    }

    uint8_t *raw = NULL;
    size_t raw_len = 0;
    if (read_file(argv[1], &raw, &raw_len) != 0) {
        fprintf(stderr, "failed to read input\n");
        return 3;
    }

    uint8_t *packed = NULL;
    size_t packed_len = 0;
    int rc = compress_lzss(raw, raw_len, &packed, &packed_len);
    if (rc != 0) {
        fprintf(stderr, "compression failed: %d\n", rc);
        free(raw);
        return 4;
    }

    uint8_t *check = NULL;
    size_t check_len = 0;
    rc = decompress_lzss(packed, packed_len, raw_len, &check, &check_len);
    if (rc != 0 || check_len != raw_len || memcmp(check, raw, raw_len) != 0) {
        fprintf(stderr, "round-trip verification failed\n");
        free(raw);
        free(packed);
        free(check);
        return 5;
    }
    free(check);

    if (write_file(argv[2], packed, packed_len) != 0) {
        fprintf(stderr, "failed to write output\n");
        free(raw);
        free(packed);
        return 6;
    }

    printf(
        "{\"raw_size\":%I64u,\"packed_size\":%I64u}\n",
        (unsigned long long)raw_len,
        (unsigned long long)packed_len);
    free(raw);
    free(packed);
    return 0;
}
