// cbm_fast.c - C accelerator for KEYINSE / 刻音色 .CBM decoding.
// Build on Windows with MinGW:
//   gcc -O3 -Wall -Wextra -shared -o cbm_fast.dll cbm_fast.c
// Build on Windows with MSVC:
//   cl /O2 /LD cbm_fast.c /Fe:cbm_fast.dll
// Build on Linux:
//   gcc -O3 -Wall -Wextra -fPIC -shared -o libcbm_fast.so cbm_fast.c
//
// Exported API is intentionally tiny for ctypes:
//   int cbm_decode_to_rgba(const uint8_t* data, size_t data_size, int flip_y,
//                          uint8_t** out_pixels, int* width, int* height, int* channels,
//                          uint32_t* packed_size, uint32_t* consumed_size,
//                          char* errbuf, size_t errbuf_size);
//   void cbm_free(void* p);

#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#if defined(_WIN32)
#define CBM_API __declspec(dllexport)
#else
#define CBM_API __attribute__((visibility("default")))
#endif

static void seterr(char* errbuf, size_t cap, const char* msg) {
    if (!errbuf || cap == 0) return;
    snprintf(errbuf, cap, "%s", msg ? msg : "unknown error");
}

static void seterrf(char* errbuf, size_t cap, const char* fmt, int a, int b, int c) {
    if (!errbuf || cap == 0) return;
    snprintf(errbuf, cap, fmt, a, b, c);
}

static uint16_t rd16le(const uint8_t* p) {
    return (uint16_t)(p[0] | ((uint16_t)p[1] << 8));
}

static uint32_t rd32le(const uint8_t* p) {
    return (uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) | ((uint32_t)p[3] << 24);
}

static int put_channel(uint8_t* dib, size_t row_stride, int channels, int x, int y, int ch, uint8_t value) {
    dib[(size_t)y * row_stride + (size_t)x * (size_t)channels + (size_t)ch] = value;
    return 0;
}

CBM_API void cbm_free(void* p) {
    free(p);
}

CBM_API int cbm_decode_to_rgba(
    const uint8_t* data,
    size_t data_size,
    int flip_y,
    uint8_t** out_pixels,
    int* width_out,
    int* height_out,
    int* channels_out,
    uint32_t* packed_size_out,
    uint32_t* consumed_size_out,
    char* errbuf,
    size_t errbuf_size
) {
    if (out_pixels) *out_pixels = NULL;
    if (width_out) *width_out = 0;
    if (height_out) *height_out = 0;
    if (channels_out) *channels_out = 0;
    if (packed_size_out) *packed_size_out = 0;
    if (consumed_size_out) *consumed_size_out = 0;

    if (!data || data_size < 10) {
        seterr(errbuf, errbuf_size, "file too small for CBM header");
        return -1;
    }
    if (!out_pixels || !width_out || !height_out || !channels_out) {
        seterr(errbuf, errbuf_size, "invalid output pointer");
        return -2;
    }

    int channels = (int)rd16le(data + 0);
    int width = (int)rd16le(data + 2);
    int height = (int)rd16le(data + 4);
    uint32_t packed_size = rd32le(data + 6);

    if (channels != 3 && channels != 4) {
        seterrf(errbuf, errbuf_size, "unsupported channels=%d; expected 3 or 4", channels, 0, 0);
        return -3;
    }
    if (width <= 0 || height <= 0) {
        seterrf(errbuf, errbuf_size, "invalid image size %dx%d", width, height, 0);
        return -4;
    }

    size_t payload_avail = data_size - 10;
    if ((size_t)packed_size > payload_avail) {
        packed_size = (uint32_t)payload_avail;
    }
    if (packed_size_out) *packed_size_out = packed_size;

    size_t row_stride = (((size_t)width * (size_t)channels + 3u) / 4u) * 4u;
    size_t dib_size = row_stride * (size_t)height;
    size_t out_size = (size_t)width * (size_t)height * (size_t)channels;

    if (height != 0 && row_stride > ((size_t)-1) / (size_t)height) {
        seterr(errbuf, errbuf_size, "image size overflow");
        return -5;
    }
    if (channels != 0 && (size_t)width > ((size_t)-1) / (size_t)height / (size_t)channels) {
        seterr(errbuf, errbuf_size, "output size overflow");
        return -6;
    }

    uint8_t* dib = (uint8_t*)calloc(1, dib_size);
    uint8_t* out = (uint8_t*)malloc(out_size);
    if (!dib || !out) {
        free(dib);
        free(out);
        seterr(errbuf, errbuf_size, "out of memory");
        return -7;
    }

    // Match the Python fallback / recovered engine behavior for 32bpp defaults.
    if (channels == 4) {
        for (int y = 0; y < height; ++y) {
            uint8_t* row = dib + (size_t)y * row_stride;
            for (int x = 0; x < width; ++x) {
                row[(size_t)x * 4u + 2u] = 0xFFu;
            }
        }
    }

    const uint8_t* payload = data + 10;
    size_t pos = 0;

    for (int ch = 0; ch < channels; ++ch) {
        for (int y = 0; y < height; ++y) {
            int x = 0;
            while (x < width) {
                if (pos + 2u > (size_t)packed_size) {
                    free(dib);
                    free(out);
                    seterrf(errbuf, errbuf_size, "truncated packet at channel=%d row=%d x=%d", ch, y, x);
                    return -8;
                }
                uint8_t base = payload[pos];
                uint8_t ctrl = payload[pos + 1u];
                pos += 2u;

                put_channel(dib, row_stride, channels, x, y, ch, base);
                x += 1;

                if (ctrl & 0x80u) {
                    int count = (int)(ctrl & 0x7Fu);
                    if (x + count > width) {
                        free(dib);
                        free(out);
                        seterrf(errbuf, errbuf_size, "repeat packet overruns row at channel=%d row=%d x=%d", ch, y, x);
                        return -9;
                    }
                    for (int k = 0; k < count; ++k) {
                        put_channel(dib, row_stride, channels, x, y, ch, base);
                        x += 1;
                    }
                } else {
                    int count = (int)ctrl;
                    size_t packed_nibbles = (size_t)((count + 1) / 2);
                    if (pos + packed_nibbles > (size_t)packed_size) {
                        free(dib);
                        free(out);
                        seterrf(errbuf, errbuf_size, "truncated nibble data at channel=%d row=%d x=%d", ch, y, x);
                        return -10;
                    }
                    if (x + count > width) {
                        free(dib);
                        free(out);
                        seterrf(errbuf, errbuf_size, "nibble packet overruns row at channel=%d row=%d x=%d", ch, y, x);
                        return -11;
                    }
                    uint8_t high = (uint8_t)(base & 0xF0u);
                    for (int k = 0; k < count; ++k) {
                        uint8_t packed = payload[pos + (size_t)(k / 2)];
                        uint8_t low = (uint8_t)((k % 2 == 0) ? (packed >> 4) : (packed & 0x0Fu));
                        put_channel(dib, row_stride, channels, x, y, ch, (uint8_t)(high | low));
                        x += 1;
                    }
                    pos += packed_nibbles;
                }
            }
        }
    }

    if (consumed_size_out) *consumed_size_out = (uint32_t)pos;

    size_t dst = 0;
    if (channels == 3) {
        if (flip_y) {
            for (int y = height - 1; y >= 0; --y) {
                const uint8_t* row = dib + (size_t)y * row_stride;
                for (int x = 0; x < width; ++x) {
                    const uint8_t* p = row + (size_t)x * 3u;
                    out[dst++] = p[2];
                    out[dst++] = p[1];
                    out[dst++] = p[0];
                }
            }
        } else {
            for (int y = 0; y < height; ++y) {
                const uint8_t* row = dib + (size_t)y * row_stride;
                for (int x = 0; x < width; ++x) {
                    const uint8_t* p = row + (size_t)x * 3u;
                    out[dst++] = p[2];
                    out[dst++] = p[1];
                    out[dst++] = p[0];
                }
            }
        }
    } else {
        if (flip_y) {
            for (int y = height - 1; y >= 0; --y) {
                const uint8_t* row = dib + (size_t)y * row_stride;
                for (int x = 0; x < width; ++x) {
                    const uint8_t* p = row + (size_t)x * 4u;
                    out[dst++] = p[2];
                    out[dst++] = p[1];
                    out[dst++] = p[0];
                    out[dst++] = p[3];
                }
            }
        } else {
            for (int y = 0; y < height; ++y) {
                const uint8_t* row = dib + (size_t)y * row_stride;
                for (int x = 0; x < width; ++x) {
                    const uint8_t* p = row + (size_t)x * 4u;
                    out[dst++] = p[2];
                    out[dst++] = p[1];
                    out[dst++] = p[0];
                    out[dst++] = p[3];
                }
            }
        }
    }

    free(dib);
    *out_pixels = out;
    *width_out = width;
    *height_out = height;
    *channels_out = channels;
    return 0;
}
