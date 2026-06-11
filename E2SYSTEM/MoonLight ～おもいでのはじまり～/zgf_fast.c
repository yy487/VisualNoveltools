// zgf_fast.c - C accelerator for MoonLight Renewal ZGF -> PNG conversion
// Build examples:
//   Windows MinGW: gcc -O3 -shared -o zgf_fast.dll zgf_fast.c
//   Linux/macOS :  gcc -O3 -fPIC -shared -o libzgf_fast.so zgf_fast.c
#include <stdint.h>
#include <stddef.h>

#if defined(_WIN32)
#define ZGF_API __declspec(dllexport)
#else
#define ZGF_API __attribute__((visibility("default")))
#endif

// ZGF stores B/G/R residual planes. The engine combines them as
//   pixel = B | (G << 8) | (R << 16)
// then applies cumulative XOR with seed 0x007F7F7F across pixels.
// Output is RGB or RGBA scanline-order bytes for PNG encoding.
ZGF_API int zgf_planes_to_pngbuf(
    const uint8_t *plane_b,
    const uint8_t *plane_g,
    const uint8_t *plane_r,
    const uint8_t *plane_a,
    uint8_t *out,
    size_t pixel_count,
    int out_channels)
{
    if (!plane_b || !plane_g || !plane_r || !out) return -1;
    if (out_channels != 3 && out_channels != 4) return -2;

    uint32_t prev = 0x007F7F7Fu;
    if (out_channels == 3) {
        for (size_t i = 0; i < pixel_count; ++i) {
            uint32_t v = ((uint32_t)plane_b[i]) |
                         ((uint32_t)plane_g[i] << 8) |
                         ((uint32_t)plane_r[i] << 16);
            v ^= prev;
            prev = v;
            out[i * 3 + 0] = (uint8_t)((v >> 16) & 0xFFu); // R
            out[i * 3 + 1] = (uint8_t)((v >>  8) & 0xFFu); // G
            out[i * 3 + 2] = (uint8_t)( v        & 0xFFu); // B
        }
    } else {
        for (size_t i = 0; i < pixel_count; ++i) {
            uint32_t v = ((uint32_t)plane_b[i]) |
                         ((uint32_t)plane_g[i] << 8) |
                         ((uint32_t)plane_r[i] << 16);
            v ^= prev;
            prev = v;
            out[i * 4 + 0] = (uint8_t)((v >> 16) & 0xFFu); // R
            out[i * 4 + 1] = (uint8_t)((v >>  8) & 0xFFu); // G
            out[i * 4 + 2] = (uint8_t)( v        & 0xFFu); // B
            out[i * 4 + 3] = plane_a ? plane_a[i] : 0xFFu; // A
        }
    }
    return 0;
}
