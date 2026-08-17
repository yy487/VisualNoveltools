use crate::archive::{Segment, DUMP_FORMAT, HEADER_SIZE, MAGIC};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub source_file: String,
    pub source_size: u64,
    pub source_sha256: String,
    pub header_values: Vec<u32>,
    pub segments: Vec<Segment>,
    pub notes: String,
}

impl Manifest {
    pub fn validate(&self, dump_dir: &Path) -> Result<(), String> {
        if self.format != DUMP_FORMAT {
            return Err(format!("unsupported manifest format: {:?}", self.format));
        }
        if self.header_values.len() != 11 {
            return Err(format!(
                "manifest header_values must contain 11 values, got {}",
                self.header_values.len()
            ));
        }
        validate_segment_plan(&self.segments)?;

        let header_path = dump_dir.join("header.bin");
        let header = fs::read(&header_path)
            .map_err(|e| format!("cannot read {}: {e}", header_path.display()))?;
        if header.len() != HEADER_SIZE || header.get(..4) != Some(MAGIC.as_slice()) {
            return Err(format!("invalid header.bin in {}", dump_dir.display()));
        }
        let actual_header_values: Vec<u32> = header
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("four bytes")))
            .collect();
        if actual_header_values != self.header_values {
            return Err("header.bin does not match manifest header_values".to_string());
        }

        for segment in &self.segments {
            validate_leaf_name(&segment.file)?;
            let payload_path = dump_dir.join(&segment.file);
            if !payload_path.is_file() {
                return Err(format!(
                    "segment payload is missing: {}",
                    payload_path.display()
                ));
            }
        }
        Ok(())
    }
}

pub fn validate_leaf_name(name: &str) -> Result<(), String> {
    let path = Path::new(name);
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => Err(format!("unsafe segment file name in manifest: {name:?}")),
    }
}

fn validate_segment_plan(segments: &[Segment]) -> Result<(), String> {
    const TAGS: [&str; 9] = [
        "CODE", "TTBL", "FTBL", "FTBL", "VTBL", "CSTR", "CDBL", "DBG_", "DBG_",
    ];
    if segments.len() < TAGS.len() || segments.len() > TAGS.len() + 1 {
        return Err(format!(
            "manifest must contain 9 SB2 segments and at most one tail, got {}",
            segments.len()
        ));
    }
    for (index, expected) in TAGS.iter().enumerate() {
        let segment = &segments[index];
        if segment.index != index || segment.tag != *expected || segment.no_tag {
            return Err(format!(
                "invalid segment {index}: tag={:?}, index={}, no_tag={}",
                segment.tag, segment.index, segment.no_tag
            ));
        }
    }
    if let Some(tail) = segments.get(TAGS.len()) {
        if tail.index != TAGS.len() || tail.tag != "TAIL" || !tail.no_tag {
            return Err("invalid opaque tail record in manifest".to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_leaf_name;

    #[test]
    fn rejects_manifest_path_traversal() {
        assert!(validate_leaf_name("../CODE.bin").is_err());
        assert!(validate_leaf_name("nested/CODE.bin").is_err());
        assert!(validate_leaf_name("CODE.bin").is_ok());
    }
}
