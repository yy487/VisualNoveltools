use crate::basic::TranslationFile;
use crate::Result;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

const MAX_ENTRIES: usize = 131;
const DECODER_BODY_SIZE: usize = 45;
const DECODER_SIZE: usize = DECODER_BODY_SIZE + 8;
const INDEX_SIZE: usize = 256;

#[derive(Debug, Clone, Default)]
pub struct TextDictionary {
    entries: Vec<[u16; 2]>,
    markers: Vec<u8>,
    source_text_bytes: usize,
    encoded_text_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextCompressionReport {
    pub method: String,
    pub dictionary_entries: usize,
    pub source_text_bytes: usize,
    pub encoded_text_bytes: usize,
    pub saved_text_bytes: usize,
    pub runtime_patch_bytes: usize,
    pub decoder_address: Option<String>,
}

impl TextDictionary {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn build(
        translations: &BTreeMap<String, TranslationFile>,
        reserved_renderer_bytes: &HashSet<u8>,
    ) -> Self {
        let messages = translations
            .values()
            .flat_map(|file| &file.entries)
            .filter(|entry| entry.kind != "print")
            .filter(|entry| {
                entry
                    .message
                    .bytes()
                    .all(|byte| byte == b' ' || (b'!'..=b'~').contains(&byte))
            })
            .map(|entry| entry.message.as_bytes().to_vec())
            .collect::<Vec<_>>();
        Self::from_messages(&messages, reserved_renderer_bytes)
    }

    fn from_messages(messages: &[Vec<u8>], reserved_renderer_bytes: &HashSet<u8>) -> Self {
        let markers = dictionary_markers(reserved_renderer_bytes);
        let source_text_bytes = messages.iter().map(Vec::len).sum();
        let mut working = messages
            .iter()
            .map(|message| {
                message
                    .iter()
                    .map(|byte| u16::from(*byte))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut entries = Vec::new();
        while entries.len() < MAX_ENTRIES && entries.len() < markers.len() {
            let mut counts: HashMap<[u16; 2], usize> = HashMap::new();
            for message in &working {
                for pair in message.windows(2) {
                    *counts.entry([pair[0], pair[1]]).or_default() += 1;
                }
            }
            let mut ranked = counts.into_iter().collect::<Vec<_>>();
            ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| right.0.cmp(&left.0)));
            let Some(candidate) = ranked.into_iter().map(|(pair, _)| pair).find(|pair| {
                working
                    .iter()
                    .map(|message| count_pair_nonoverlapping(message, *pair))
                    .sum::<usize>()
                    > 5
            }) else {
                break;
            };
            let symbol = 0x100 + entries.len() as u16;
            for message in &mut working {
                *message = replace_pair(message, candidate, symbol);
            }
            entries.push(candidate);
        }
        let encoded_text_bytes = working.iter().map(Vec::len).sum();
        Self {
            entries,
            markers,
            source_text_bytes,
            encoded_text_bytes,
        }
    }

    pub fn encode_text(&self, text: &str) -> Result<Vec<u8>> {
        if !text
            .bytes()
            .all(|byte| byte == b' ' || (b'!'..=b'~').contains(&byte))
        {
            return Err("消息正文只支持半角 ASCII 20h–7Eh；控制字节由脚本结构自动保留".into());
        }
        let mut symbols = text.bytes().map(u16::from).collect::<Vec<_>>();
        for (index, entry) in self.entries.iter().enumerate() {
            symbols = replace_pair(&symbols, *entry, 0x100 + index as u16);
        }
        symbols
            .into_iter()
            .map(|symbol| {
                if symbol >= 0x100 {
                    Ok(self.markers[usize::from(symbol - 0x100)])
                } else {
                    let byte = symbol as u8;
                    if byte == b' ' {
                        Ok(byte)
                    } else {
                        Ok(0x86 + (byte - b'!'))
                    }
                }
            })
            .collect()
    }

    pub fn runtime_patch(&self, decoder_address: u16) -> Result<Vec<u8>> {
        if self.entries.is_empty() {
            return Ok(Vec::new());
        }
        let index_address = decoder_address
            .checked_add(DECODER_SIZE as u16)
            .ok_or("文本字典索引地址溢出")?;
        let pointers_address = index_address
            .checked_add(INDEX_SIZE as u16)
            .ok_or("文本字典指针地址溢出")?;
        let literals_address = pointers_address
            .checked_add((self.entries.len() * 2) as u16)
            .ok_or("文本字典正文地址溢出")?;

        let mut output = decoder_code(decoder_address, index_address, pointers_address);
        if output.len() != DECODER_SIZE {
            return Err(format!(
                "文本字典解码器为 {} 字节，预期 {DECODER_SIZE}",
                output.len()
            ));
        }
        let mut index = [0xFFu8; INDEX_SIZE];
        for (entry_index, marker) in self.markers[..self.entries.len()].iter().enumerate() {
            index[usize::from(*marker)] = entry_index as u8;
        }
        output.extend_from_slice(&index);

        let expanded_entries = (0..self.entries.len())
            .map(|index| self.expanded_entry(index))
            .collect::<Vec<_>>();
        let mut address = literals_address;
        for entry in &expanded_entries {
            output.extend_from_slice(&address.to_le_bytes());
            address = address
                .checked_add(u16::try_from(entry.len() + 1).map_err(|_| "文本字典条目长度溢出")?)
                .ok_or("文本字典条目地址溢出")?;
        }
        for entry in expanded_entries {
            output.extend(entry);
            output.push(0);
        }
        Ok(output)
    }

    fn expanded_entry(&self, index: usize) -> Vec<u8> {
        fn append(dictionary: &TextDictionary, symbol: u16, output: &mut Vec<u8>) {
            if symbol >= 0x100 {
                for child in dictionary.entries[usize::from(symbol - 0x100)] {
                    append(dictionary, child, output);
                }
            } else if symbol == u16::from(b' ') {
                output.push(b' ');
            } else {
                output.push(0x86 + (symbol as u8 - b'!'));
            }
        }

        let mut output = Vec::new();
        for symbol in self.entries[index] {
            append(self, symbol, &mut output);
        }
        output
    }

    pub fn report(&self, decoder_address: Option<u16>) -> Result<TextCompressionReport> {
        let runtime_patch_bytes = decoder_address
            .map(|address| self.runtime_patch(address).map(|patch| patch.len()))
            .transpose()?
            .unwrap_or(0);
        Ok(TextCompressionReport {
            method: "one-byte byte-pair dictionary with flat M3DATA runtime expansions".into(),
            dictionary_entries: self.entries.len(),
            source_text_bytes: self.source_text_bytes,
            encoded_text_bytes: self.encoded_text_bytes,
            saved_text_bytes: self.source_text_bytes - self.encoded_text_bytes,
            runtime_patch_bytes,
            decoder_address: decoder_address.map(|address| format!("0x{address:04X}")),
        })
    }
}

fn count_pair_nonoverlapping(message: &[u16], pair: [u16; 2]) -> usize {
    let mut count = 0;
    let mut offset = 0;
    while offset + 1 < message.len() {
        if message[offset] == pair[0] && message[offset + 1] == pair[1] {
            count += 1;
            offset += 2;
        } else {
            offset += 1;
        }
    }
    count
}

fn replace_pair(message: &[u16], pair: [u16; 2], symbol: u16) -> Vec<u16> {
    let mut output = Vec::with_capacity(message.len());
    let mut offset = 0;
    while offset < message.len() {
        if offset + 1 < message.len()
            && message[offset] == pair[0]
            && message[offset + 1] == pair[1]
        {
            output.push(symbol);
            offset += 2;
        } else {
            output.push(message[offset]);
            offset += 1;
        }
    }
    output
}

fn dictionary_markers(reserved_renderer_bytes: &HashSet<u8>) -> Vec<u8> {
    (1u8..=0xFE)
        .filter(|byte| {
            *byte != b' '
                && *byte != b'"'
                // N88's LOAD/RUN relinker parses numeric tokens even inside strings.
                && !(0x0B..=0x1F).contains(byte)
                && !(0x86..=0xE3).contains(byte)
                && !reserved_renderer_bytes.contains(byte)
        })
        .take(MAX_ENTRIES)
        .collect()
}

fn decoder_code(decoder_address: u16, index_address: u16, pointers_address: u16) -> Vec<u8> {
    let [index_low, index_high] = index_address.to_le_bytes();
    let [pointers_low, pointers_high] = pointers_address.to_le_bytes();
    let [native_low, native_high] = (decoder_address + DECODER_BODY_SIZE as u16).to_le_bytes();
    vec![
        0xE5, // PUSH HL; preserve the source character pointer for literal fallback
        0x7E, // LD A,(HL)
        0x6F, // LD L,A
        0x26,
        0x00, // LD H,0
        0x11,
        index_low,
        index_high, // LD DE,index
        0x19,       // ADD HL,DE
        0x7E,       // LD A,(HL)
        0xFE,
        0xFF, // CP FFh
        0x20,
        0x04, // JR NZ,dictionary token
        0xE1, // POP HL; BD93h expects the original source character pointer
        0xC3,
        native_low,
        native_high, // JP native renderer trampoline, bypassing the BD93h hook
        0xE1,        // dictionary token: discard the saved source pointer
        0x87,        // ADD A,A
        0x6F,        // LD L,A
        0x26,
        0x00, // LD H,0
        0x30,
        0x01, // JR NC,+1
        0x24, // INC H
        0x11,
        pointers_low,
        pointers_high, // LD DE,pointers
        0x19,          // ADD HL,DE
        0x5E,          // LD E,(HL)
        0x23,          // INC HL
        0x56,          // LD D,(HL)
        0xEB,          // EX DE,HL
        0x7E,          // loop: LD A,(HL)
        0xA7,          // AND A
        0xC8,          // RET Z
        0xE5,          // PUSH HL
        0xCD,
        native_low,
        native_high, // CALL native renderer; entries are fully expanded at build time
        0xE1,        // POP HL
        0x23,        // INC HL
        0x18,
        0xF5, // JR loop
        // Replay the five original bytes replaced by the common BD93h hook.
        0x3E,
        0x00, // LD A,0
        0x32,
        0x8F,
        0xBE, // LD (BE8Fh),A
        0xC3,
        0x98,
        0xBD, // JP BD98h
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_set_is_unique_and_avoids_native_controls() {
        let reserved = [
            b'!', b'*', b'+', b'/', b'0', b'9', b'<', b'=', b'>', b'?', b'B', b'G', b'W', b'\\',
        ]
        .into_iter()
        .collect::<HashSet<_>>();
        let markers = dictionary_markers(&reserved);
        let mut sorted = markers.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert!(!markers.is_empty());
        assert!(markers.len() <= MAX_ENTRIES);
        assert_eq!(sorted.len(), markers.len());
        for forbidden in [
            0x00, b' ', b'!', b'"', b'*', b'+', b'/', b'0', b'9', b'<', b'=', b'>', b'?', b'B',
            b'G', b'W', b'\\', 0x86, 0xE3, 0xFF,
        ] {
            assert!(!markers.contains(&forbidden));
        }
        assert!(markers.iter().all(|byte| !(0x0B..=0x1F).contains(byte)));
    }

    #[test]
    fn dictionary_round_trip_shape() {
        let messages = vec![b"the thing and the other thing".to_vec(); 12];
        let dictionary = TextDictionary::from_messages(&messages, &HashSet::new());
        let encoded = dictionary
            .encode_text("the thing and the other thing")
            .unwrap();
        assert!(encoded.len() < messages[0].len());
        let patch = dictionary.runtime_patch(0xF200).unwrap();
        assert!(patch.len() > DECODER_SIZE + INDEX_SIZE);
        for index in 0..dictionary.entries.len() {
            let expanded = dictionary.expanded_entry(index);
            assert!(!expanded
                .iter()
                .any(|byte| dictionary.markers[..dictionary.entries.len()].contains(byte)));
        }
    }

    #[test]
    fn literal_fallback_restores_original_hl() {
        let code = decoder_code(0xF200, 0xF235, 0xF335);
        assert_eq!(code.len(), DECODER_SIZE);
        assert_eq!(code[0], 0xE5); // PUSH HL
        assert_eq!(&code[12..18], &[0x20, 0x04, 0xE1, 0xC3, 0x2D, 0xF2]);
        assert_eq!(&code[38..41], &[0xCD, 0x2D, 0xF2]);
        assert_eq!(
            &code[45..],
            &[0x3E, 0x00, 0x32, 0x8F, 0xBE, 0xC3, 0x98, 0xBD]
        );
    }

    #[test]
    fn all_dictionary_symbols_are_safe_for_n88_relinking() {
        // Exercise every marker, including entries beyond the first ten. The
        // former 1Ch/10h pair made the ROM relinker revisit its input forever.
        let markers = dictionary_markers(&HashSet::new());
        let mut entries = Vec::new();
        let mut message = Vec::new();
        for index in 0..markers.len() {
            let pair = [
                u16::from(b'A' + (index / 26) as u8),
                u16::from(b'a' + (index % 26) as u8),
            ];
            entries.push(pair);
            message.extend([pair[0] as u8, pair[1] as u8, b' ']);
        }
        let dictionary = TextDictionary {
            entries,
            markers: markers.clone(),
            ..TextDictionary::default()
        };
        let encoded = dictionary
            .encode_text(std::str::from_utf8(&message).unwrap())
            .unwrap();
        assert_eq!(encoded.len(), markers.len() * 2);
        assert_eq!(
            encoded.iter().step_by(2).copied().collect::<Vec<_>>(),
            markers
        );
        assert!(encoded.iter().all(|byte| !(0x0B..=0x1F).contains(byte)));
    }
}
