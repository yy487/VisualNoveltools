/*
 * MMO C acceleration core for Doukyuusei/AI5WIN MMO -> PNG workflow.
 *
 * Exports:
 *   int mmo_decode_rgb_fast(const uint8_t *src, size_t src_len,
 *                           uint32_t width, uint32_t height,
 *                           uint8_t *dst_rgb, size_t dst_len,
 *                           size_t *used_src,
 *                           char *errbuf, size_t errbuf_len);
 *
 * Function:
 *   - LZSS decompresses the RGB stream.
 *   - Restores the MMO delta coding in BGR byte order.
 *   - Converts BGR to standard RGB for Pillow Image.frombytes("RGB", ...).
 *
 * This file is intentionally self-contained so it can be built either as a DLL
 * with MSVC/MinGW or as a .so on Linux for local verification.
 */

#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#if defined(_WIN32) || defined(__CYGWIN__)
#  define MMO_API __declspec(dllexport)
#else
#  define MMO_API __attribute__((visibility("default")))
#endif

static void set_err(char *errbuf, size_t errbuf_len, const char *msg) {
    if (errbuf && errbuf_len > 0) {
#if defined(_MSC_VER)
        strncpy_s(errbuf, errbuf_len, msg, _TRUNCATE);
#else
        snprintf(errbuf, errbuf_len, "%s", msg);
#endif
    }
}

static int lzss_decompress_mmo(const uint8_t *src, size_t src_len,
                               uint8_t *out, size_t expected_size,
                               size_t *used_src,
                               char *errbuf, size_t errbuf_len) {
    size_t pos = 0;
    size_t out_pos = 0;
    uint32_t flags = 0;
    uint8_t ring[0x1000];
    uint32_t rpos = 0xFEE;

    memset(ring, 0, sizeof(ring));

    while (out_pos < expected_size) {
        flags >>= 1;
        if ((flags & 0x100u) == 0) {
            if (pos >= src_len) {
                set_err(errbuf, errbuf_len, "compressed stream ended while reading flags");
                return -1;
            }
            flags = (uint32_t)src[pos++] | 0xFF00u;
        }

        if (flags & 1u) {
            uint8_t b;
            if (pos >= src_len) {
                set_err(errbuf, errbuf_len, "compressed stream ended while reading literal");
                return -2;
            }
            b = src[pos++];
            out[out_pos++] = b;
            ring[rpos] = b;
            rpos = (rpos + 1u) & 0xFFFu;
        } else {
            uint8_t b0, b1;
            uint32_t offset, count, i;
            if (pos + 2 > src_len) {
                set_err(errbuf, errbuf_len, "compressed stream ended while reading copy token");
                return -3;
            }
            b0 = src[pos++];
            b1 = src[pos++];
            offset = (uint32_t)b0 | (((uint32_t)b1 & 0xF0u) << 4);
            count = ((uint32_t)b1 & 0x0Fu) + 3u;
            for (i = 0; i < count && out_pos < expected_size; i++) {
                uint8_t b = ring[(offset + i) & 0xFFFu];
                out[out_pos++] = b;
                ring[rpos] = b;
                rpos = (rpos + 1u) & 0xFFFu;
            }
        }
    }

    if (used_src) {
        *used_src = pos;
    }
    return 0;
}

static void restore_delta_bgr(uint8_t *buf, uint32_t width, uint32_t height) {
    uint32_t x, y;
    size_t stride = (size_t)width * 3u;

    /* 第一行：按左侧像素横向累加。 */
    for (x = 1; x < width; x++) {
        size_t i = (size_t)x * 3u;
        buf[i + 0] = (uint8_t)(buf[i + 0] + buf[i - 3]);
        buf[i + 1] = (uint8_t)(buf[i + 1] + buf[i - 2]);
        buf[i + 2] = (uint8_t)(buf[i + 2] + buf[i - 1]);
    }

    /* 后续行：按上一行同列纵向累加。 */
    for (y = 1; y < height; y++) {
        size_t row = (size_t)y * stride;
        size_t prev = row - stride;
        for (x = 0; x < width; x++) {
            size_t i = row + (size_t)x * 3u;
            size_t j = prev + (size_t)x * 3u;
            buf[i + 0] = (uint8_t)(buf[i + 0] + buf[j + 0]);
            buf[i + 1] = (uint8_t)(buf[i + 1] + buf[j + 1]);
            buf[i + 2] = (uint8_t)(buf[i + 2] + buf[j + 2]);
        }
    }
}

MMO_API int mmo_decode_rgb_fast(const uint8_t *src, size_t src_len,
                                uint32_t width, uint32_t height,
                                uint8_t *dst_rgb, size_t dst_len,
                                size_t *used_src,
                                char *errbuf, size_t errbuf_len) {
    size_t expected;
    uint8_t *tmp;
    int rc;
    size_t i;

    if (!src || !dst_rgb || width == 0 || height == 0) {
        set_err(errbuf, errbuf_len, "invalid argument");
        return -10;
    }
    if ((size_t)width > ((size_t)-1) / (size_t)height / 3u) {
        set_err(errbuf, errbuf_len, "image size overflow");
        return -11;
    }
    expected = (size_t)width * (size_t)height * 3u;
    if (dst_len < expected) {
        set_err(errbuf, errbuf_len, "destination buffer too small");
        return -12;
    }

    tmp = (uint8_t *)malloc(expected);
    if (!tmp) {
        set_err(errbuf, errbuf_len, "malloc failed");
        return -13;
    }

    rc = lzss_decompress_mmo(src, src_len, tmp, expected, used_src, errbuf, errbuf_len);
    if (rc != 0) {
        free(tmp);
        return rc;
    }

    restore_delta_bgr(tmp, width, height);

    /* BGR -> RGB，Pillow 侧可以直接 Image.frombytes("RGB", ...)。 */
    for (i = 0; i < expected; i += 3u) {
        dst_rgb[i + 0] = tmp[i + 2];
        dst_rgb[i + 1] = tmp[i + 1];
        dst_rgb[i + 2] = tmp[i + 0];
    }

    free(tmp);
    return 0;
}

MMO_API const char *mmo_fast_version(void) {
    return "mmo_fast_v1";
}
