#include <stdint.h>
#include <stddef.h>

#ifdef _WIN32
#define API __declspec(dllexport)
#else
#define API __attribute__((visibility("default")))
#endif

/*
 * Engine channel memory order is B, G, R for 24bpp TYP1.
 * PNG/Pillow RGB order is R, G, B.
 */
API void typ1_bgr_to_rgb24(const uint8_t *b, const uint8_t *g, const uint8_t *r,
                           uint8_t *rgb, size_t pixels) {
    for (size_t i = 0; i < pixels; ++i) {
        size_t j = i * 3;
        rgb[j + 0] = r[i];
        rgb[j + 1] = g[i];
        rgb[j + 2] = b[i];
    }
}
