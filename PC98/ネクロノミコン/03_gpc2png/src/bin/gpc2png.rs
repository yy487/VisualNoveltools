use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct GpcImage {
    width: usize,
    height: usize,
    offset_x: i16,
    offset_y: i16,
    interleaving: usize,
    palette: Vec<[u8; 3]>,
    pixels: Vec<u8>,
}

#[derive(Debug)]
struct GpcError(String);

impl std::fmt::Display for GpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for GpcError {}

fn u16_at(data: &[u8], offset: usize) -> Result<u16, GpcError> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| GpcError(format!("truncated u16 at 0x{offset:X}")))?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn u32_at(data: &[u8], offset: usize) -> Result<u32, GpcError> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| GpcError(format!("truncated u32 at 0x{offset:X}")))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_gpc(data: &[u8]) -> Result<GpcImage, GpcError> {
    const MAGIC: &[u8] = b"PC98)GPCFILE   \0";
    if data.len() < 0x64 || data.get(..MAGIC.len()) != Some(MAGIC) {
        return Err(GpcError("not a PC-98 GPCFILE".to_owned()));
    }

    let interleaving = usize::from(u16_at(data, 0x10)?);
    if interleaving == 0 {
        return Err(GpcError("invalid zero interleaving".to_owned()));
    }
    let palette_offset = usize::try_from(u32_at(data, 0x14)?)
        .map_err(|_| GpcError("palette offset does not fit usize".to_owned()))?;
    let info_offset = usize::try_from(u32_at(data, 0x18)?)
        .map_err(|_| GpcError("info offset does not fit usize".to_owned()))?;
    let data_offset = info_offset
        .checked_add(0x10)
        .ok_or_else(|| GpcError("data offset overflow".to_owned()))?;
    let width = usize::from(u16_at(data, info_offset)?);
    let height = usize::from(u16_at(data, info_offset + 2)?);
    let block_size = usize::try_from(u32_at(data, info_offset + 4)?)
        .map_err(|_| GpcError("block size does not fit usize".to_owned()))?;
    let planes = u16_at(data, info_offset + 8)?;
    if planes != 4 {
        return Err(GpcError(format!("unsupported plane count {planes}")));
    }
    let offset_x = u16_at(data, info_offset + 0xA)? as i16;
    let offset_y = u16_at(data, info_offset + 0xC)? as i16;
    if width == 0 || height == 0 {
        return Err(GpcError("zero image dimensions".to_owned()));
    }
    if block_size < 0x10 || data_offset > data.len() || block_size > data.len() - info_offset {
        return Err(GpcError(format!(
            "invalid image block: offset=0x{info_offset:X}, size=0x{block_size:X}, file=0x{:X}",
            data.len()
        )));
    }

    let palette_count = usize::from(u16_at(data, palette_offset)?);
    let palette_element_size = u16_at(data, palette_offset + 2)?;
    if palette_count != 16 || palette_element_size != 2 {
        return Err(GpcError(format!(
            "unsupported palette: count={palette_count}, element_size={palette_element_size}"
        )));
    }
    let palette_end = palette_offset
        .checked_add(4 + palette_count * 2)
        .ok_or_else(|| GpcError("palette size overflow".to_owned()))?;
    if palette_end > data.len() {
        return Err(GpcError("truncated palette".to_owned()));
    }
    let mut palette = Vec::with_capacity(palette_count);
    for index in 0..palette_count {
        let value = u16_at(data, palette_offset + 4 + index * 2)?;
        let r = ((value >> 4) & 0xF) as u8 * 0x11;
        let g = ((value >> 8) & 0xF) as u8 * 0x11;
        let b = (value & 0xF) as u8 * 0x11;
        palette.push([r, g, b]);
    }

    let plane_stride = width.div_ceil(8);
    let row_size = plane_stride
        .checked_mul(4)
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| GpcError("row size overflow".to_owned()))?;
    let unpacked_size = row_size
        .checked_mul(height)
        .ok_or_else(|| GpcError("unpacked size overflow".to_owned()))?;
    if data_offset > info_offset + block_size || data_offset > data.len() {
        return Err(GpcError("image data starts outside block".to_owned()));
    }
    let compressed = &data[data_offset..info_offset + block_size];
    let mut rows = vec![0u8; unpacked_size];
    unpack_data(compressed, &mut rows)?;
    restore_data(&mut rows, row_size, height);
    let pixels = convert_to_packed_pixels(&rows, width, height, plane_stride, interleaving)?;

    Ok(GpcImage {
        width,
        height,
        offset_x,
        offset_y,
        interleaving,
        palette,
        pixels,
    })
}

fn unpack_data(input: &[u8], output: &mut [u8]) -> Result<(), GpcError> {
    let mut src = 0usize;
    let mut dst = 0usize;
    let mut control = 0u8;
    let mut control_mask = 0u8;
    while dst < output.len() {
        if control_mask == 0 {
            control = *input
                .get(src)
                .ok_or_else(|| GpcError(format!("compressed stream ended at 0x{src:X}")))?;
            src += 1;
            control_mask = 0x80;
        }
        if control & control_mask != 0 {
            let command = *input
                .get(src)
                .ok_or_else(|| GpcError(format!("missing command byte at 0x{src:X}")))?;
            src += 1;
            for mask in [0x80u8, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01] {
                if dst >= output.len() {
                    break;
                }
                if command & mask != 0 {
                    output[dst] = *input
                        .get(src)
                        .ok_or_else(|| GpcError(format!("missing literal byte at 0x{src:X}")))?;
                    src += 1;
                }
                dst += 1;
            }
        } else {
            dst = dst
                .checked_add(8)
                .ok_or_else(|| GpcError("unpacked position overflow".to_owned()))?;
            if dst > output.len() {
                // The original decoder treats a final skip as filling the
                // remaining zero bytes and stops once the image is full.
                dst = output.len();
            }
        }
        control_mask >>= 1;
    }
    Ok(())
}

fn restore_data(data: &mut [u8], stride: usize, height: usize) {
    let mut row_start = 0usize;
    for y in 0..height {
        let interleave = usize::from(data[row_start]);
        if interleave != 0 {
            let mut last = 0u8;
            for lane in 0..interleave {
                let mut position = 1 + lane;
                while position < stride {
                    let index = row_start + position;
                    data[index] ^= last;
                    last = data[index];
                    position += interleave;
                }
            }
        }
        if y > 0 {
            let length = (stride - 1) & !3;
            let previous = row_start - stride;
            for x in 1..=length {
                data[row_start + x] ^= data[previous + x];
            }
        }
        row_start += stride;
    }
}

fn convert_to_packed_pixels(
    input: &[u8],
    width: usize,
    height: usize,
    plane_stride: usize,
    interleaving: usize,
) -> Result<Vec<u8>, GpcError> {
    let output_stride = plane_stride * 4;
    let output_len = output_stride
        .checked_mul(height)
        .ok_or_else(|| GpcError("pixel buffer size overflow".to_owned()))?;
    let mut output = vec![0u8; output_len];
    let interleaving_step = output_stride
        .checked_mul(interleaving)
        .ok_or_else(|| GpcError("interleaving step overflow".to_owned()))?;
    let mut source_row = 1usize;
    let mut destination_row = 0usize;
    let mut wrap = 0usize;
    for _ in 0..height {
        if destination_row >= output.len() {
            wrap += 1;
            destination_row = output_stride * wrap;
        }
        let p0 = source_row;
        let p1 = p0 + plane_stride;
        let p2 = p1 + plane_stride;
        let p3 = p2 + plane_stride;
        let mut destination = destination_row;
        for x in 0..plane_stride {
            let b0 = *input
                .get(p0 + x)
                .ok_or_else(|| GpcError("plane 0 overrun".to_owned()))?;
            let b1 = *input
                .get(p1 + x)
                .ok_or_else(|| GpcError("plane 1 overrun".to_owned()))?;
            let b2 = *input
                .get(p2 + x)
                .ok_or_else(|| GpcError("plane 2 overrun".to_owned()))?;
            let b3 = *input
                .get(p3 + x)
                .ok_or_else(|| GpcError("plane 3 overrun".to_owned()))?;
            for shift in [0u8, 2, 4, 6] {
                let first = ((b0 << shift) & 0x80) >> 3
                    | ((b1 << shift) & 0x80) >> 2
                    | ((b2 << shift) & 0x80) >> 1
                    | ((b3 << shift) & 0x80);
                let second = ((b0 << shift) & 0x40) >> 6
                    | ((b1 << shift) & 0x40) >> 5
                    | ((b2 << shift) & 0x40) >> 4
                    | ((b3 << shift) & 0x40) >> 3;
                output[destination] = first | second;
                destination += 1;
            }
        }
        source_row = p3 + plane_stride + 1;
        destination_row += interleaving_step;
    }
    let expected_stride = width.div_ceil(2);
    if output_stride < expected_stride {
        return Err(GpcError(
            "decoded stride is smaller than image width".to_owned(),
        ));
    }
    Ok(output)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in bytes {
        a = (a + u32::from(byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

fn write_chunk(output: &mut Vec<u8>, kind: &[u8; 4], payload: &[u8]) {
    output.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    output.extend_from_slice(kind);
    output.extend_from_slice(payload);
    let mut crc_input = Vec::with_capacity(4 + payload.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(payload);
    output.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

fn encode_png(image: &GpcImage) -> Vec<u8> {
    let packed_stride = image.width.div_ceil(2);
    let mut raw = Vec::with_capacity((packed_stride * 2 + 1) * image.height);
    let source_stride = image.pixels.len() / image.height;
    for y in 0..image.height {
        raw.push(0);
        let row = &image.pixels[y * source_stride..y * source_stride + packed_stride];
        for (packed_index, &packed) in row.iter().enumerate() {
            raw.push(packed >> 4);
            if packed_index * 2 + 1 < image.width {
                raw.push(packed & 0x0F);
            }
        }
    }

    let mut zlib = Vec::with_capacity(raw.len() + raw.len() / 65_535 * 5 + 16);
    zlib.extend_from_slice(&[0x78, 0x01]);
    let mut offset = 0usize;
    while offset < raw.len() {
        let end = (offset + 65_535).min(raw.len());
        let final_block = end == raw.len();
        zlib.push(if final_block { 1 } else { 0 });
        let length = (end - offset) as u16;
        zlib.extend_from_slice(&length.to_le_bytes());
        zlib.extend_from_slice(&(!length).to_le_bytes());
        zlib.extend_from_slice(&raw[offset..end]);
        offset = end;
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1A\n");
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(image.width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(image.height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 3, 0, 0, 0]);
    write_chunk(&mut png, b"IHDR", &ihdr);
    let palette_bytes: Vec<u8> = image
        .palette
        .iter()
        .flat_map(|rgb| rgb.iter().copied())
        .collect();
    write_chunk(&mut png, b"PLTE", &palette_bytes);
    write_chunk(&mut png, b"IDAT", &zlib);
    write_chunk(&mut png, b"IEND", &[]);
    png
}

fn collect_inputs(path: &Path) -> io::Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_owned()]);
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            files.extend(collect_inputs(&entry_path)?);
        } else if entry_path
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| x.eq_ignore_ascii_case("gpc"))
        {
            files.push(entry_path);
        }
    }
    files.sort();
    Ok(files)
}

fn output_path(input: &Path, input_root: &Path, output: Option<&Path>, multiple: bool) -> PathBuf {
    if let Some(output) = output {
        if multiple || output.is_dir() {
            let relative = input.strip_prefix(input_root).unwrap_or(input);
            return output.join(relative).with_extension("png");
        }
        return output.to_owned();
    }
    input.with_extension("png")
}

fn print_help() {
    println!("gpc2png - decode PC-98 GPCFILE images to indexed PNG");
    println!("Usage: gpc2png <file-or-directory> [--output <path>]");
    println!("A directory is searched recursively for .GPC files. Existing outputs are never overwritten.");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }
    let input_root = PathBuf::from(&args[0]);
    let output = args
        .windows(2)
        .find_map(|pair| (pair[0] == "--output").then(|| PathBuf::from(&pair[1])));
    let inputs = collect_inputs(&input_root)?;
    if inputs.is_empty() {
        return Err(format!("no GPC files found under {}", input_root.display()).into());
    }
    let multiple = inputs.len() > 1 || input_root.is_dir();
    let mut converted = 0usize;
    let mut warnings = 0usize;
    for input in inputs {
        let bytes = fs::read(&input)?;
        match read_gpc(&bytes) {
            Ok(image) => {
                let destination = output_path(&input, &input_root, output.as_deref(), multiple);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                if destination.exists() {
                    return Err(format!("refusing to overwrite {}", destination.display()).into());
                }
                fs::write(&destination, encode_png(&image))?;
                converted += 1;
                println!(
                    "[gpc2png] {} -> {} ({}x{}, offset {},{}, interleaving {})",
                    input.display(),
                    destination.display(),
                    image.width,
                    image.height,
                    image.offset_x,
                    image.offset_y,
                    image.interleaving
                );
            }
            Err(error) => {
                warnings += 1;
                eprintln!("[gpc2png] warning: {}: {error}", input.display());
            }
        }
    }
    println!("[gpc2png] converted={converted} warnings={warnings}");
    if converted == 0 {
        return Err("no GPC image was converted".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known_vector() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn adler32_known_vector() {
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }
}
