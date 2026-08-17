use crate::manifest::Manifest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const MAGIC: [u8; 4] = *b"SB2 ";
pub const HEADER_SIZE: usize = 0x2c;
pub const DUMP_FORMAT: &str = "AGSI_SB2_DUMP_V2";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Segment {
    pub index: usize,
    pub tag: String,
    pub file: String,
    pub tag_offset: u64,
    pub data_offset: u64,
    pub size: u64,
    #[serde(default, skip_serializing_if = "is_false")]
    pub no_tag: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cstr_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cstr_table_offset_in_file: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cstr_pool_offset_in_file: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cstr_pool_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cstr_table_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cstr_entries_total_size: Option<u64>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone)]
pub struct ParsedArchive {
    pub header: Vec<u8>,
    pub header_values: Vec<u32>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct InspectReport {
    pub file_size: u64,
    pub sha256: String,
    pub parsed: ParsedArchive,
}

#[derive(Debug, Clone)]
pub struct UnpackReport {
    pub extracted_files: usize,
    pub source_size: u64,
    pub source_sha256: String,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PackReport {
    pub packed_files: usize,
    pub output_bytes: u64,
    pub output_sha256: String,
    pub byte_equal: Option<bool>,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct VerifyReport {
    pub archive_size: u64,
    pub rebuilt_size: u64,
    pub archive_sha256: String,
    pub rebuilt_sha256: String,
    pub byte_equal: bool,
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read(&mut self, size: usize, context: &str) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(size)
            .ok_or_else(|| format!("offset overflow while reading {context}"))?;
        if end > self.data.len() {
            return Err(format!(
                "truncated {context}: offset=0x{:x}, size=0x{size:x}, file_size=0x{:x}",
                self.pos,
                self.data.len()
            ));
        }
        let result = &self.data[self.pos..end];
        self.pos = end;
        Ok(result)
    }

    fn read_u32(&mut self, context: &str) -> Result<u32, String> {
        let bytes = self.read(4, context)?;
        Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }

    fn expect_tag(&mut self, expected: &str) -> Result<usize, String> {
        let offset = self.pos;
        let actual = self.read(4, expected)?;
        if actual != expected.as_bytes() {
            return Err(format!(
                "tag mismatch at 0x{offset:x}: got={actual:?}, expected={:?}",
                expected.as_bytes()
            ));
        }
        Ok(offset)
    }
}

pub fn parse_archive(data: &[u8]) -> Result<ParsedArchive, String> {
    let mut reader = Reader::new(data);
    let header = reader.read(HEADER_SIZE, "SB2 header")?.to_vec();
    if header[..4] != MAGIC {
        return Err(format!("not an SB2 archive: magic={:?}", &header[..4]));
    }

    let header_values: Vec<u32> = header
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("four bytes")))
        .collect();
    validate_runtime_header(&header_values)?;
    let mut segments = Vec::with_capacity(10);

    take_segment(&mut reader, &mut segments, "CODE", "CODE.bin", |r| {
        r.read(header_values[3] as usize, "CODE payload")?;
        Ok(None)
    })?;
    take_segment(&mut reader, &mut segments, "TTBL", "TTBL.bin", |r| {
        skip_ttbl(r, header_values[5])?;
        Ok(None)
    })?;
    take_segment(&mut reader, &mut segments, "FTBL", "FTBL_0.bin", |r| {
        skip_ftbl(r, header_values[6], "FTBL_0")?;
        Ok(None)
    })?;
    take_segment(&mut reader, &mut segments, "FTBL", "FTBL_1.bin", |r| {
        skip_ftbl(r, header_values[7], "FTBL_1")?;
        Ok(None)
    })?;
    take_segment(&mut reader, &mut segments, "VTBL", "VTBL.bin", |r| {
        let size = checked_mul(header_values[8] as usize, 12, "VTBL size")?;
        r.read(size, "VTBL payload")?;
        Ok(None)
    })?;
    take_segment(&mut reader, &mut segments, "CSTR", "CSTR.bin", |r| {
        skip_cstr(r, header_values[9]).map(Some)
    })?;
    take_segment(&mut reader, &mut segments, "CDBL", "CDBL.bin", |r| {
        let size = checked_mul(header_values[10] as usize, 8, "CDBL size")?;
        r.read(size, "CDBL payload")?;
        Ok(None)
    })?;
    take_segment(&mut reader, &mut segments, "DBG_", "DBG_0.bin", |r| {
        skip_dbg_files(r)?;
        Ok(None)
    })?;
    take_segment(&mut reader, &mut segments, "DBG_", "DBG_1.bin", |r| {
        skip_dbg_lines(r)?;
        Ok(None)
    })?;

    if reader.pos < data.len() {
        let size = data.len() - reader.pos;
        segments.push(Segment {
            index: segments.len(),
            tag: "TAIL".to_string(),
            file: "TAIL.bin".to_string(),
            tag_offset: reader.pos as u64,
            data_offset: reader.pos as u64,
            size: size as u64,
            no_tag: true,
            cstr_count: None,
            cstr_table_offset_in_file: None,
            cstr_pool_offset_in_file: None,
            cstr_pool_size: None,
            cstr_table_size: None,
            cstr_entries_total_size: None,
        });
    }

    Ok(ParsedArchive {
        header,
        header_values,
        segments,
    })
}

fn validate_runtime_header(values: &[u32]) -> Result<(), String> {
    if values[1] != 0 || values[2] != 0 {
        return Err(format!(
            "unsupported SB2 version fields: h1={}, h2={}",
            values[1], values[2]
        ));
    }
    if values[5] < 4 {
        return Err(format!(
            "invalid TTBL count: runtime requires at least 4, got {}",
            values[5]
        ));
    }
    for index in [3_usize, 4, 5, 6, 7, 8, 9, 10] {
        if values[index] > i32::MAX as u32 {
            return Err(format!(
                "header field h[{index}] exceeds the runtime's signed range: {}",
                values[index]
            ));
        }
    }
    Ok(())
}

fn take_segment<F>(
    reader: &mut Reader<'_>,
    segments: &mut Vec<Segment>,
    tag: &str,
    file: &str,
    skip: F,
) -> Result<(), String>
where
    F: FnOnce(&mut Reader<'_>) -> Result<Option<CstrDetails>, String>,
{
    let tag_offset = reader.expect_tag(tag)?;
    let data_offset = reader.pos;
    let details = skip(reader)?;
    let size = reader.pos - data_offset;
    segments.push(Segment {
        index: segments.len(),
        tag: tag.to_string(),
        file: file.to_string(),
        tag_offset: tag_offset as u64,
        data_offset: data_offset as u64,
        size: size as u64,
        no_tag: false,
        cstr_count: details.as_ref().map(|x| x.count),
        cstr_table_offset_in_file: details.as_ref().map(|x| x.table_offset as u64),
        cstr_pool_offset_in_file: details.as_ref().map(|x| x.pool_offset as u64),
        cstr_pool_size: details.as_ref().map(|x| x.pool_size as u64),
        cstr_table_size: details.as_ref().map(|x| x.table_size as u64),
        cstr_entries_total_size: details.as_ref().map(|x| x.entries_total_size),
    });
    Ok(())
}

#[derive(Debug)]
struct CstrDetails {
    count: u32,
    table_offset: usize,
    pool_offset: usize,
    pool_size: usize,
    table_size: usize,
    entries_total_size: u64,
}

fn skip_ttbl(reader: &mut Reader<'_>, count: u32) -> Result<(), String> {
    for record in 0..count {
        reader.read(4, &format!("TTBL[{record}] type"))?;
        let member_count = reader.read_u32(&format!("TTBL[{record}] member count"))?;
        for member in 0..member_count {
            reader.read(4, &format!("TTBL[{record}].member[{member}] kind"))?;
            let _dimension_count =
                reader.read_u32(&format!("TTBL[{record}].member[{member}] dimensions"))?;
            reader.read(16, &format!("TTBL[{record}].member[{member}] metadata"))?;
            reader.read(4, &format!("TTBL[{record}].member[{member}] flags"))?;
        }
    }
    Ok(())
}

fn skip_ftbl(reader: &mut Reader<'_>, count: u32, label: &str) -> Result<(), String> {
    for record in 0..count {
        let name_size = reader.read_u32(&format!("{label}[{record}] name size"))? as usize;
        reader.read(name_size, &format!("{label}[{record}] name"))?;
        reader.read(12, &format!("{label}[{record}] metadata"))?;
    }
    Ok(())
}

fn skip_cstr(reader: &mut Reader<'_>, count: u32) -> Result<CstrDetails, String> {
    let table_offset = reader.pos;
    let table_size = checked_mul(count as usize, 8, "CSTR table size")?;
    let table = reader.read(table_size, "CSTR offset/size table")?;
    let pool_offset = reader.pos;
    let mut max_end = 0_u64;
    let mut entries_total_size = 0_u64;
    for (index, entry) in table.chunks_exact(8).enumerate() {
        let offset = u32::from_le_bytes(entry[..4].try_into().expect("four bytes")) as u64;
        let size = u32::from_le_bytes(entry[4..].try_into().expect("four bytes")) as u64;
        let end = offset
            .checked_add(size)
            .ok_or_else(|| format!("CSTR[{index}] offset+size overflow"))?;
        max_end = max_end.max(end);
        entries_total_size = entries_total_size
            .checked_add(size)
            .ok_or_else(|| "CSTR total size overflow".to_string())?;
    }
    if max_end > entries_total_size {
        return Err(format!(
            "CSTR entry points beyond the runtime-sized pool: max_end={max_end}, sum_sizes={entries_total_size}"
        ));
    }
    let pool_size = usize::try_from(entries_total_size)
        .map_err(|_| format!("CSTR pool is too large for this platform: {entries_total_size}"))?;
    reader.read(pool_size, "CSTR string pool")?;
    Ok(CstrDetails {
        count,
        table_offset,
        pool_offset,
        pool_size,
        table_size,
        entries_total_size,
    })
}

fn skip_dbg_files(reader: &mut Reader<'_>) -> Result<(), String> {
    let count = reader.read_u32("DBG file count")?;
    for index in 0..count {
        let name_size = reader.read_u32(&format!("DBG file[{index}] name size"))? as usize;
        reader.read(name_size, &format!("DBG file[{index}] name"))?;
    }
    Ok(())
}

fn skip_dbg_lines(reader: &mut Reader<'_>) -> Result<(), String> {
    let count = reader.read_u32("DBG line count")?;
    let size = checked_mul(count as usize, 12, "DBG line table size")?;
    reader.read(size, "DBG line table")?;
    Ok(())
}

fn checked_mul(left: usize, right: usize, context: &str) -> Result<usize, String> {
    left.checked_mul(right)
        .ok_or_else(|| format!("integer overflow while calculating {context}"))
}

pub fn inspect_archive(path: &Path) -> Result<InspectReport, String> {
    let data = fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let parsed = parse_archive(&data)?;
    Ok(InspectReport {
        file_size: data.len() as u64,
        sha256: sha256_hex(&data),
        parsed,
    })
}

pub fn unpack_archive(
    input: &Path,
    output_dir: &Path,
    overwrite: bool,
) -> Result<UnpackReport, String> {
    if output_dir.exists() {
        if !overwrite {
            return Err(format!(
                "output directory already exists: {}",
                output_dir.display()
            ));
        }
        ensure_managed_dump(output_dir)?;
    }
    let data = fs::read(input).map_err(|e| format!("cannot read {}: {e}", input.display()))?;
    let parsed = parse_archive(&data)?;
    let source_sha256 = sha256_hex(&data);
    let source_file = input
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("source.sb")
        .to_string();
    let stage = staging_path(output_dir, "unpack")?;
    if stage.exists() {
        return Err(format!("staging path already exists: {}", stage.display()));
    }
    fs::create_dir_all(&stage)
        .map_err(|e| format!("cannot create staging directory {}: {e}", stage.display()))?;

    let write_result = (|| {
        fs::write(stage.join("header.bin"), &parsed.header)
            .map_err(|e| format!("cannot write staged header: {e}"))?;
        for segment in &parsed.segments {
            let start = segment.data_offset as usize;
            let end = start
                .checked_add(segment.size as usize)
                .ok_or_else(|| "segment range overflow".to_string())?;
            fs::write(stage.join(&segment.file), &data[start..end])
                .map_err(|e| format!("cannot write staged {}: {e}", segment.file))?;
        }
        let manifest = Manifest {
            format: DUMP_FORMAT.to_string(),
            source_file,
            source_size: data.len() as u64,
            source_sha256: source_sha256.clone(),
            header_values: parsed.header_values.clone(),
            segments: parsed.segments.clone(),
            notes: "Segment payload files omit their 4-byte tags; pack restores tags in manifest order. TAIL.bin, when present, is opaque trailing data.".to_string(),
        };
        let mut json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("cannot serialize manifest: {e}"))?;
        json.push('\n');
        fs::write(stage.join("manifest.json"), json.as_bytes())
            .map_err(|e| format!("cannot write staged manifest: {e}"))?;
        Ok::<(), String>(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }

    if output_dir.exists() {
        if let Err(error) = fs::remove_dir_all(output_dir) {
            let _ = fs::remove_dir_all(&stage);
            return Err(format!(
                "cannot replace managed output {}: {error}",
                output_dir.display()
            ));
        }
    }
    fs::rename(&stage, output_dir).map_err(|e| {
        let _ = fs::remove_dir_all(&stage);
        format!(
            "cannot publish output directory {}: {e}",
            output_dir.display()
        )
    })?;

    Ok(UnpackReport {
        extracted_files: parsed.segments.len() + 2,
        source_size: data.len() as u64,
        source_sha256,
        output_dir: output_dir.to_path_buf(),
    })
}

pub fn pack_archive(
    dump_dir: &Path,
    output: &Path,
    overwrite: bool,
    compare_original: Option<&Path>,
) -> Result<PackReport, String> {
    let manifest = load_manifest(dump_dir)?;
    let data = build_archive_from_dump(dump_dir)?;
    parse_archive(&data)
        .map_err(|e| format!("rebuilt archive failed structural validation: {e}"))?;
    if output.exists() && overwrite {
        let existing = fs::read(output)
            .map_err(|e| format!("cannot inspect existing output {}: {e}", output.display()))?;
        if sha256_hex(&existing) == manifest.source_sha256 {
            return Err(format!(
                "refusing to overwrite a file identical to the original source archive: {}",
                output.display()
            ));
        }
    }
    write_atomic_file(output, &data, overwrite)?;
    let byte_equal = if let Some(original) = compare_original {
        let original_data = fs::read(original)
            .map_err(|e| format!("cannot read comparison archive {}: {e}", original.display()))?;
        Some(original_data == data)
    } else {
        None
    };
    Ok(PackReport {
        packed_files: manifest.segments.len(),
        output_bytes: data.len() as u64,
        output_sha256: sha256_hex(&data),
        byte_equal,
        output: output.to_path_buf(),
    })
}

pub fn verify_archive_against_dump(
    archive: &Path,
    dump_dir: &Path,
) -> Result<VerifyReport, String> {
    let archive_data =
        fs::read(archive).map_err(|e| format!("cannot read {}: {e}", archive.display()))?;
    parse_archive(&archive_data)?;
    let rebuilt = build_archive_from_dump(dump_dir)?;
    parse_archive(&rebuilt)
        .map_err(|e| format!("rebuilt archive failed structural validation: {e}"))?;
    Ok(VerifyReport {
        archive_size: archive_data.len() as u64,
        rebuilt_size: rebuilt.len() as u64,
        archive_sha256: sha256_hex(&archive_data),
        rebuilt_sha256: sha256_hex(&rebuilt),
        byte_equal: archive_data == rebuilt,
    })
}

fn build_archive_from_dump(dump_dir: &Path) -> Result<Vec<u8>, String> {
    let manifest = load_manifest(dump_dir)?;
    manifest.validate(dump_dir)?;
    let mut header = fs::read(dump_dir.join("header.bin"))
        .map_err(|e| format!("cannot read header.bin: {e}"))?;

    let code = fs::read(dump_dir.join(&manifest.segments[0].file))
        .map_err(|e| format!("cannot read CODE payload: {e}"))?;
    let code_size = u32::try_from(code.len())
        .map_err(|_| format!("CODE payload is too large: {} bytes", code.len()))?;
    header[0x0c..0x10].copy_from_slice(&code_size.to_le_bytes());

    let total_payload: u64 = manifest.segments.iter().map(|x| x.size).sum();
    let estimated = HEADER_SIZE as u64
        + 4 * manifest.segments.iter().filter(|x| !x.no_tag).count() as u64
        + total_payload;
    let capacity = usize::try_from(estimated).unwrap_or(header.len());
    let mut out = Vec::with_capacity(capacity);
    out.extend_from_slice(&header);
    for segment in &manifest.segments {
        if !segment.no_tag {
            out.extend_from_slice(segment.tag.as_bytes());
        }
        let payload_path = dump_dir.join(&segment.file);
        let payload = fs::read(&payload_path)
            .map_err(|e| format!("cannot read {}: {e}", payload_path.display()))?;
        out.extend_from_slice(&payload);
    }
    Ok(out)
}

fn load_manifest(dump_dir: &Path) -> Result<Manifest, String> {
    let path = dump_dir.join("manifest.json");
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {} as UTF-8: {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("invalid {}: {e}", path.display()))
}

fn ensure_managed_dump(path: &Path) -> Result<(), String> {
    let manifest = load_manifest(path).map_err(|_| {
        format!(
            "refusing to overwrite non-managed directory {}; choose another output or remove it manually",
            path.display()
        )
    })?;
    if manifest.format != DUMP_FORMAT {
        return Err(format!(
            "refusing to overwrite directory with foreign manifest: {}",
            path.display()
        ));
    }
    Ok(())
}

fn staging_path(target: &Path, label: &str) -> Result<PathBuf, String> {
    let parent = target
        .parent()
        .ok_or_else(|| format!("output has no parent: {}", target.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("cannot create output parent {}: {e}", parent.display()))?;
    let name = target
        .file_name()
        .and_then(|x| x.to_str())
        .ok_or_else(|| format!("output has an invalid file name: {}", target.display()))?;
    Ok(parent.join(format!(".{name}.{label}.{}.tmp", std::process::id())))
}

fn write_atomic_file(output: &Path, data: &[u8], overwrite: bool) -> Result<(), String> {
    if output.exists() && !overwrite {
        return Err(format!("output file already exists: {}", output.display()));
    }
    let stage = staging_path(output, "pack")?;
    if stage.exists() {
        return Err(format!("staging file already exists: {}", stage.display()));
    }
    let mut file = fs::File::create(&stage)
        .map_err(|e| format!("cannot create staging file {}: {e}", stage.display()))?;
    if let Err(error) = file.write_all(data).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&stage);
        return Err(format!(
            "cannot write staging file {}: {error}",
            stage.display()
        ));
    }
    drop(file);
    if output.exists() {
        fs::remove_file(output)
            .map_err(|e| format!("cannot replace output {}: {e}", output.display()))?;
    }
    fs::rename(&stage, output).map_err(|e| {
        let _ = fs::remove_file(&stage);
        format!("cannot publish output {}: {e}", output.display())
    })
}

fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

#[cfg(test)]
mod tests {
    use super::{parse_archive, HEADER_SIZE, MAGIC};

    fn supported_header(cstr_count: u32) -> Vec<u8> {
        let mut header = vec![0_u8; HEADER_SIZE];
        header[..4].copy_from_slice(&MAGIC);
        header[0x14..0x18].copy_from_slice(&4_u32.to_le_bytes());
        header[0x24..0x28].copy_from_slice(&cstr_count.to_le_bytes());
        header
    }

    fn append_through_cstr_tag(out: &mut Vec<u8>) {
        out.extend_from_slice(b"CODE");
        out.extend_from_slice(b"TTBL");
        out.extend_from_slice(&[0_u8; 32]);
        for tag in [b"FTBL", b"FTBL", b"VTBL", b"CSTR"] {
            out.extend_from_slice(tag);
        }
    }

    fn append_empty_suffix(out: &mut Vec<u8>) {
        out.extend_from_slice(b"CDBL");
        out.extend_from_slice(b"DBG_");
        out.extend_from_slice(&0_u32.to_le_bytes());
        out.extend_from_slice(b"DBG_");
        out.extend_from_slice(&0_u32.to_le_bytes());
    }

    fn minimal_archive() -> Vec<u8> {
        let mut out = supported_header(0);
        append_through_cstr_tag(&mut out);
        append_empty_suffix(&mut out);
        out
    }

    #[test]
    fn parses_zero_count_vtbl_as_empty_payload() {
        let parsed = parse_archive(&minimal_archive()).expect("minimal archive should parse");
        let vtbl = parsed.segments.iter().find(|x| x.tag == "VTBL").unwrap();
        assert_eq!(vtbl.size, 0);
        assert_eq!(parsed.segments.len(), 9);
    }

    #[test]
    fn parses_runtime_sized_vtbl_record() {
        let mut out = supported_header(0);
        out[0x20..0x24].copy_from_slice(&1_u32.to_le_bytes());
        out.extend_from_slice(b"CODE");
        out.extend_from_slice(b"TTBL");
        out.extend_from_slice(&[0_u8; 32]);
        out.extend_from_slice(b"FTBLFTBLVTBL");
        out.extend_from_slice(&[0_u8; 12]);
        out.extend_from_slice(b"CSTR");
        append_empty_suffix(&mut out);
        let parsed = parse_archive(&out).expect("VTBL record should parse");
        let vtbl = parsed.segments.iter().find(|x| x.tag == "VTBL").unwrap();
        assert_eq!(vtbl.size, 12);
    }

    #[test]
    fn parses_runtime_sized_ttbl_member() {
        let mut out = supported_header(0);
        out.extend_from_slice(b"CODETTBL");
        out.extend_from_slice(&1_u32.to_le_bytes());
        out.extend_from_slice(&1_u32.to_le_bytes());
        out.extend_from_slice(&[0_u8; 28]);
        for kind in [2_u32, 4, 8] {
            out.extend_from_slice(&kind.to_le_bytes());
            out.extend_from_slice(&0_u32.to_le_bytes());
        }
        out.extend_from_slice(b"FTBLFTBLVTBLCSTR");
        append_empty_suffix(&mut out);
        let parsed = parse_archive(&out).expect("TTBL member should parse");
        let ttbl = parsed.segments.iter().find(|x| x.tag == "TTBL").unwrap();
        assert_eq!(ttbl.size, 60);
    }

    #[test]
    fn rejects_truncated_archive() {
        let mut archive = minimal_archive();
        archive.pop();
        assert!(parse_archive(&archive).is_err());
    }

    #[test]
    fn rejects_wrong_segment_order() {
        let mut archive = minimal_archive();
        archive[HEADER_SIZE] = b'X';
        assert!(parse_archive(&archive).is_err());
    }

    #[test]
    fn parses_cstr_offset_size_pool() {
        let mut out = supported_header(1);
        append_through_cstr_tag(&mut out);
        out.extend_from_slice(&0_u32.to_le_bytes());
        out.extend_from_slice(&2_u32.to_le_bytes());
        out.extend_from_slice(b"A\0");
        append_empty_suffix(&mut out);
        let parsed = parse_archive(&out).expect("CSTR archive should parse");
        let cstr = parsed.segments.iter().find(|x| x.tag == "CSTR").unwrap();
        assert_eq!(cstr.size, 10);
        assert_eq!(cstr.cstr_pool_size, Some(2));
    }

    #[test]
    fn rejects_cstr_entry_outside_runtime_sized_pool() {
        let mut out = supported_header(1);
        append_through_cstr_tag(&mut out);
        out.extend_from_slice(&10_u32.to_le_bytes());
        out.extend_from_slice(&2_u32.to_le_bytes());
        out.extend_from_slice(b"A\0");
        append_empty_suffix(&mut out);
        assert!(parse_archive(&out).is_err());
    }

    #[test]
    fn rejects_header_the_runtime_would_reject() {
        let mut archive = minimal_archive();
        archive[0x14..0x18].copy_from_slice(&3_u32.to_le_bytes());
        assert!(parse_archive(&archive).is_err());
    }
}
