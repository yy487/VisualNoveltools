//! Native PC-88 text glyphs used by the original Random House scripts.
//!
//! These bytes are visible CGROM characters, not two-byte KANJI1 carriers.
//! Keeping the mapping bidirectional lets extracted source glyphs survive an
//! edited message without requiring the selected TrueType font to contain them.

pub fn native_glyph_for_byte(byte: u8) -> Option<char> {
    Some(match byte {
        0x5C => '¥',
        0x80 => '▁',
        0x82 => '▃',
        0x83 => '▄',
        0x85 => '▆',
        0x87 => '█',
        0x89 => '▎',
        0x8A => '▍',
        0x8D => '▊',
        0x8E => '▉',
        0x8F => '┼',
        0x9F => '✓',
        0xE5 => '◣',
        0xE9 => '♥',
        0xEA => '◆',
        0xF0 => '╳',
        0xF3 => '年',
        _ => return None,
    })
}

pub fn native_byte_for_glyph(glyph: char) -> Option<u8> {
    Some(match glyph {
        '¥' => 0x5C,
        '▁' => 0x80,
        '▃' => 0x82,
        '▄' => 0x83,
        '▆' => 0x85,
        '█' => 0x87,
        '▎' => 0x89,
        '▍' => 0x8A,
        '▊' => 0x8D,
        '▉' => 0x8E,
        '┼' => 0x8F,
        '✓' => 0x9F,
        '◣' => 0xE5,
        '♥' => 0xE9,
        '◆' => 0xEA,
        '╳' => 0xF0,
        '年' => 0xF3,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_visible_glyph_mapping_is_bidirectional() {
        for byte in [
            0x5C, 0x80, 0x82, 0x83, 0x85, 0x87, 0x89, 0x8A, 0x8D, 0x8E, 0x8F, 0x9F, 0xE5, 0xE9,
            0xEA, 0xF0, 0xF3,
        ] {
            let glyph = native_glyph_for_byte(byte).unwrap();
            assert_eq!(native_byte_for_glyph(glyph), Some(byte));
        }
    }
}
