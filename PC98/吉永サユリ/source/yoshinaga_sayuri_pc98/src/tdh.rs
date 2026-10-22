use crate::{decode_text, encode_text, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
enum Node {
    Leaf(u16),
    Branch(usize, usize),
}

struct Bits<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl Bits<'_> {
    fn read(&mut self, count: usize) -> Result<u32> {
        let mut value = 0;
        for _ in 0..count {
            let byte = *self
                .bytes
                .get(self.position / 8)
                .ok_or("TDH bitstream truncated")?;
            value = (value << 1) | u32::from((byte >> (7 - self.position % 8)) & 1);
            self.position += 1;
        }
        Ok(value)
    }
}

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
    length: usize,
}
impl Writer {
    fn bit(&mut self, value: bool) {
        if self.length.is_multiple_of(8) {
            self.bytes.push(0);
        }
        if value {
            self.bytes[self.length / 8] |= 1 << (7 - self.length % 8);
        }
        self.length += 1;
    }
    fn number(&mut self, value: u32, count: usize) {
        for shift in (0..count).rev() {
            self.bit(value & (1 << shift) != 0);
        }
    }
}

fn tree(bits: &mut Bits<'_>, nodes: &mut Vec<Node>, depth: usize) -> Result<usize> {
    if depth > 128 || nodes.len() >= 8191 {
        return Err("TDH Huffman tree exceeds native limits".into());
    }
    let index = nodes.len();
    nodes.push(Node::Leaf(0));
    nodes[index] = if bits.read(1)? == 0 {
        Node::Leaf(bits.read(16)? as u16)
    } else {
        let left = tree(bits, nodes, depth + 1)?;
        let right = tree(bits, nodes, depth + 1)?;
        Node::Branch(left, right)
    };
    Ok(index)
}

fn word(bytes: &[u8], offset: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        bytes
            .get(offset..offset + 2)
            .ok_or("TDH word truncated")?
            .try_into()
            .unwrap(),
    ))
}
fn dword(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or("TDH dword truncated")?
            .try_into()
            .unwrap(),
    ))
}

pub struct Tdh {
    original: Vec<u8>,
    dictionary: Vec<u16>,
    nodes: Vec<Node>,
    table_start: usize,
    stream_start: usize,
    used_bits: usize,
    pub texts: Vec<Vec<u8>>,
}
impl Tdh {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let stream_start =
            usize::try_from(dword(bytes, 0)?).map_err(|_| "TDH header overflow")? + 4;
        if stream_start > bytes.len() || stream_start - 4 >= 65536 {
            return Err("TDH header exceeds input or 16-bit native header addressing".into());
        }
        let dictionary_count = usize::from(word(bytes, 4)?);
        if dictionary_count == 0 || dictionary_count > 4096 {
            return Err("TDH dictionary must contain 1..4096 symbols".into());
        }
        let dictionary = (0..dictionary_count)
            .map(|i| word(bytes, 6 + i * 2))
            .collect::<Result<Vec<_>>>()?;
        let mut bits = Bits {
            bytes: &bytes[..stream_start],
            position: (6 + dictionary_count * 2) * 8,
        };
        let mut nodes = Vec::new();
        tree(&mut bits, &mut nodes, 0)?;
        if !matches!(nodes[0], Node::Branch(..)) {
            return Err("TDH native decoder requires a branch at the Huffman root".into());
        }
        // 19766 fetches the next byte eagerly, even at an exact bit boundary.
        let table_start = bits.position / 8 + 1;
        let count =
            usize::try_from(dword(bytes, table_start)?).map_err(|_| "TDH text count overflow")?;
        if count == 0 || count > 65535 || table_start + 4 + count * 4 != stream_start {
            return Err("TDH text table does not fill the declared header".into());
        }
        let stream = &bytes[stream_start..];
        let mut texts = Vec::with_capacity(count);
        let mut used_bits = 0;
        for index in 0..count {
            let offset = dword(bytes, table_start + 4 + index * 4)? as usize;
            let mut bits = Bits {
                bytes: stream,
                position: offset,
            };
            let size = bits.read(11)? as usize;
            let mut text = Vec::with_capacity(size);
            while text.len() < size {
                let mut node = 0;
                loop {
                    match nodes[node] {
                        Node::Branch(left, right) => {
                            node = if bits.read(1)? == 0 { left } else { right }
                        }
                        Node::Leaf(leaf) => {
                            let code = *dictionary
                                .get(usize::from(leaf))
                                .ok_or("TDH leaf outside dictionary")?;
                            if code > 255 {
                                text.push((code >> 8) as u8);
                            }
                            text.push(code as u8);
                            break;
                        }
                    }
                }
            }
            if text.len() != size || bits.position - offset + offset % 8 > 8191 {
                return Err(format!(
                    "TDH entry {index}: invalid length or compressed text exceeds 1 KB reader"
                ));
            }
            decode_text(&text, true).map_err(|e| format!("TDH entry {index}: {e}"))?;
            used_bits = used_bits.max(bits.position);
            texts.push(text);
        }
        let archive = Self {
            original: bytes.to_vec(),
            dictionary,
            nodes,
            table_start,
            stream_start,
            used_bits,
            texts,
        };
        archive.codes()?;
        Ok(archive)
    }

    pub fn messages(&self) -> Result<Vec<String>> {
        self.texts
            .iter()
            .map(|text| decode_text(text, true))
            .collect()
    }

    fn codes(&self) -> Result<BTreeMap<u16, Vec<bool>>> {
        fn visit(
            archive: &Tdh,
            index: usize,
            path: &mut Vec<bool>,
            codes: &mut BTreeMap<u16, Vec<bool>>,
        ) -> Result<()> {
            match archive.nodes[index] {
                Node::Leaf(leaf) => {
                    let code = *archive
                        .dictionary
                        .get(usize::from(leaf))
                        .ok_or("TDH leaf outside dictionary")?;
                    codes.entry(code).or_insert_with(|| path.clone());
                }
                Node::Branch(left, right) => {
                    path.push(false);
                    visit(archive, left, path, codes)?;
                    path.pop();
                    path.push(true);
                    visit(archive, right, path, codes)?;
                    path.pop();
                }
            }
            Ok(())
        }
        let mut codes = BTreeMap::new();
        visit(self, 0, &mut Vec::new(), &mut codes)?;
        Ok(codes)
    }

    pub fn rebuild(&self, messages: &[String]) -> Result<Vec<u8>> {
        let texts = messages
            .iter()
            .enumerate()
            .map(|(i, s)| encode_text(s, true).map_err(|e| format!("TDH entry {i}: {e}")))
            .collect::<Result<Vec<_>>>()?;
        self.rebuild_encoded(&texts)
    }

    pub fn rebuild_encoded(&self, texts: &[Vec<u8>]) -> Result<Vec<u8>> {
        if texts.len() != self.texts.len() {
            return Err("TDH text count changed".into());
        }
        for (index, text) in texts.iter().enumerate() {
            if text.len() > 2047 {
                return Err(format!(
                    "TDH entry {index}: {} bytes exceeds the 11-bit limit 2047",
                    text.len()
                ));
            }
            decode_text(text, true).map_err(|e| format!("TDH entry {index}: {e}"))?;
        }
        if texts == self.texts.as_slice() {
            return Ok(self.original.clone());
        }
        let mut edited = Self {
            original: Vec::new(),
            dictionary: self.dictionary.clone(),
            nodes: self.nodes.clone(),
            table_start: 0,
            stream_start: 0,
            used_bits: 0,
            texts: Vec::new(),
        };
        let old_codes = self.codes()?;
        let mut unknown = BTreeSet::new();
        for text in texts {
            for code in symbols(text)? {
                if !old_codes.contains_key(&code) {
                    unknown.insert(code);
                }
            }
        }
        if edited.dictionary.len() + unknown.len() > 4096 {
            return Err("TDH dictionary exceeds 4096 symbols".into());
        }
        let mut prefix = if unknown.is_empty() {
            self.original[4..self.table_start + 4].to_vec()
        } else {
            let first = edited.dictionary.len();
            edited.dictionary.extend(unknown);
            fn balanced(nodes: &mut Vec<Node>, first: usize, end: usize) -> usize {
                let index = nodes.len();
                nodes.push(Node::Leaf(first as u16));
                if end - first > 1 {
                    let mid = first + (end - first) / 2;
                    let left = balanced(nodes, first, mid);
                    let right = balanced(nodes, mid, end);
                    nodes[index] = Node::Branch(left, right);
                }
                index
            }
            let right = balanced(&mut edited.nodes, first, edited.dictionary.len());
            let root = edited.nodes.len();
            edited.nodes.push(Node::Branch(0, right));
            // Keep the original tree intact beneath a new root; dictionary IDs retain meaning.
            let mut remapped = Vec::new();
            fn copy(nodes: &[Node], index: usize, result: &mut Vec<Node>) -> usize {
                let position = result.len();
                result.push(Node::Leaf(0));
                result[position] = match nodes[index] {
                    Node::Leaf(leaf) => Node::Leaf(leaf),
                    Node::Branch(left, right) => {
                        Node::Branch(copy(nodes, left, result), copy(nodes, right, result))
                    }
                };
                position
            }
            copy(&edited.nodes, root, &mut remapped);
            edited.nodes = remapped;
            let mut prefix = (edited.dictionary.len() as u16).to_le_bytes().to_vec();
            for code in &edited.dictionary {
                prefix.extend(code.to_le_bytes());
            }
            let mut writer = Writer::default();
            fn serialize(nodes: &[Node], index: usize, writer: &mut Writer) {
                match nodes[index] {
                    Node::Leaf(leaf) => {
                        writer.bit(false);
                        writer.number(u32::from(leaf), 16);
                    }
                    Node::Branch(left, right) => {
                        writer.bit(true);
                        serialize(nodes, left, writer);
                        serialize(nodes, right, writer);
                    }
                }
            }
            serialize(&edited.nodes, 0, &mut writer);
            if writer.length.is_multiple_of(8) {
                writer.bytes.push(0);
            }
            prefix.extend(writer.bytes);
            prefix.extend((texts.len() as u32).to_le_bytes());
            prefix
        };
        let codes = edited.codes()?;
        let mut stream = Writer::default();
        for (index, text) in texts.iter().enumerate() {
            if text.len() > 2047 {
                return Err(format!(
                    "TDH entry {index}: {} bytes exceeds the 11-bit limit 2047",
                    text.len()
                ));
            }
            let start = stream.length;
            prefix.extend(
                u32::try_from(start)
                    .map_err(|_| "TDH bit offset exceeds u32")?
                    .to_le_bytes(),
            );
            stream.number(text.len() as u32, 11);
            for code in symbols(text)? {
                for &bit in codes.get(&code).ok_or("TDH missing encoding code")? {
                    stream.bit(bit);
                }
            }
            if stream.length - start + start % 8 > 8191 {
                return Err(format!(
                    "TDH entry {index}: compressed entry exceeds the native 1 KB read window"
                ));
            }
        }
        if prefix.len() >= 65536 {
            return Err("TDH rebuilt header exceeds 16-bit native addressing".into());
        }
        // Preserve the original unconsumed tail bits, including opaque trailing data.
        let original_stream = &self.original[self.stream_start..];
        for position in self.used_bits..original_stream.len() * 8 {
            stream.bit(original_stream[position / 8] & (1 << (7 - position % 8)) != 0);
        }
        let mut bytes = (prefix.len() as u32).to_le_bytes().to_vec();
        bytes.extend(prefix);
        bytes.extend(stream.bytes);
        if Self::parse(&bytes)?.texts != texts {
            return Err("TDH rebuild verification failed".into());
        }
        Ok(bytes)
    }
}

fn symbols(bytes: &[u8]) -> Result<Vec<u16>> {
    let mut symbols = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let first = bytes[i];
        i += 1;
        if (0x81..=0x9f).contains(&first) || (0xe0..=0xfc).contains(&first) {
            let next = *bytes.get(i).ok_or("truncated CP932 character")?;
            i += 1;
            symbols.push(u16::from(first) * 256 + u16::from(next));
        } else {
            symbols.push(u16::from(first));
        }
    }
    Ok(symbols)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(depth: usize) -> Vec<u8> {
        let mut tree = Writer::default();
        for _ in 0..depth {
            tree.bit(true);
        }
        tree.bit(false);
        tree.number(0, 16);
        for _ in 0..depth {
            tree.bit(false);
            tree.number(1, 16);
        }
        if tree.length.is_multiple_of(8) {
            tree.bytes.push(0);
        }
        let mut prefix = vec![2, 0, b'A', 0, b'B', 0];
        prefix.extend(tree.bytes);
        prefix.extend(2u32.to_le_bytes());
        let mut stream = Writer::default();
        prefix.extend(0u32.to_le_bytes());
        stream.number(1, 11);
        for _ in 0..depth {
            stream.bit(false);
        }
        prefix.extend((stream.length as u32).to_le_bytes());
        stream.number(1, 11);
        stream.bit(true);
        let mut result = (prefix.len() as u32).to_le_bytes().to_vec();
        result.extend(prefix);
        result.extend(stream.bytes);
        result.extend([0x59, 0xa6]);
        result
    }
    #[test]
    fn variable_lengths_new_dictionary_symbol_and_noop() {
        let original = fixture(1);
        let archive = Tdh::parse(&original).unwrap();
        assert_eq!(archive.messages().unwrap(), ["A", "B"]);
        assert_eq!(
            archive.rebuild(&archive.messages().unwrap()).unwrap(),
            original
        );
        let edited = vec!["A\n校AAAA".to_string(), String::new()];
        let rebuilt = archive.rebuild(&edited).unwrap();
        assert_eq!(Tdh::parse(&rebuilt).unwrap().messages().unwrap(), edited);
        assert_eq!(Tdh::parse(&rebuilt).unwrap().texts.len(), 2);
    }
    #[test]
    fn rejects_native_length_window_and_corrupt_data() {
        let archive = Tdh::parse(&fixture(10)).unwrap();
        assert!(archive
            .rebuild(&["A".repeat(2048), "B".into()])
            .unwrap_err()
            .contains("2047"));
        assert!(archive
            .rebuild(&["A".repeat(1000), "B".into()])
            .unwrap_err()
            .contains("1 KB"));
        assert!(Tdh::parse(&fixture(1)[..8]).is_err());
        assert!(archive.rebuild(&["\0".into(), "B".into()]).is_err());
    }
}
