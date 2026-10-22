use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const SECTOR_SIZE: usize = 0x100;
pub const WORKSPACE_FORMAT: &str = "system3-dri-dat-unpack-workspace-v1";
const ROUTE_TERMINATOR: [u8; 2] = [0x1A, 0x00];

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RoutingEntry {
    pub logical_id: usize,
    pub archive_id: u8,
    pub archive_letter: char,
    pub local_member: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MemberInfo {
    pub local_member: usize,
    pub start_sector: u16,
    pub end_sector: u16,
    pub offset: usize,
    pub size: usize,
    pub sha256: String,
    pub file: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedArchive {
    pub sector_pointers: Vec<u16>,
    pub routing_entries: Vec<RoutingEntry>,
    pub route_table_offset: usize,
    pub route_table_size: usize,
    pub route_terminator_offset: usize,
    pub members: Vec<MemberInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ArchiveManifest {
    pub source_file: String,
    pub source_bytes: usize,
    pub source_sha256: String,
    pub output_dir: String,
    pub sector_size: usize,
    pub sector_pointers: Vec<u16>,
    pub route_table_offset: usize,
    pub route_table_size: usize,
    pub route_terminator_offset: usize,
    pub route_table_file: String,
    pub header_file: String,
    pub routing_entries: Vec<RoutingEntry>,
    pub members: Vec<MemberInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WorkspaceManifest {
    pub _format: String,
    pub tool_version: String,
    pub role_paths: RolePaths,
    pub archives: Vec<ArchiveManifest>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RolePaths {
    pub archive_roots: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct UnpackReport {
    pub archives: usize,
    pub members: usize,
    pub member_bytes: u64,
    pub routing_entries: usize,
    pub output_root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct PackReport {
    pub bytes: Vec<u8>,
    pub members: usize,
    pub changed_members: usize,
    pub source_bytes: usize,
    pub output_bytes: usize,
}

#[derive(Debug)]
struct PreparedArchive {
    bytes: Vec<u8>,
    output_dir: String,
    manifest: ArchiveManifest,
}

pub fn parse_archive(bytes: &[u8]) -> Result<ParsedArchive> {
    if bytes.len() < SECTOR_SIZE * 3 {
        return Err(format!(
            "文件过小: 至少需要 0x{:X} 字节，实际 0x{:X}",
            SECTOR_SIZE * 3,
            bytes.len()
        ));
    }
    if !bytes.len().is_multiple_of(SECTOR_SIZE) {
        return Err(format!("文件大小不是 0x100 对齐: 0x{:X}", bytes.len()));
    }

    let end_sector_usize = bytes
        .len()
        .checked_div(SECTOR_SIZE)
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| "计算文件尾扇区时溢出".to_string())?;
    let end_sector = u16::try_from(end_sector_usize)
        .map_err(|_| format!("文件过大，尾扇区超出 u16: {end_sector_usize}"))?;

    let first = read_u16(bytes, 0)?;
    if first != 2 {
        return Err(format!(
            "不是已确认的 DRI DAT: index[0] 应为 2，实际为 {first}"
        ));
    }

    let mut sector_pointers = vec![first];
    let mut previous = first;
    let mut found_end = false;
    for index in 1..(SECTOR_SIZE / 2) {
        let value = read_u16(bytes, index * 2)?;
        if value <= previous {
            return Err(format!(
                "扇区指针不递增: index[{index}]={value}, previous={previous}"
            ));
        }
        if value > end_sector {
            return Err(format!(
                "扇区指针越过文件尾: index[{index}]={value}, end={end_sector}"
            ));
        }
        sector_pointers.push(value);
        previous = value;
        if value == end_sector {
            found_end = true;
            break;
        }
    }
    if !found_end {
        return Err(format!("首扇区内未找到命中文件尾的终止指针 {end_sector}"));
    }
    if sector_pointers.len() < 3 {
        return Err("索引没有定义任何物理成员".to_string());
    }

    let route_table_offset = pointer_offset(sector_pointers[0])?;
    let first_member_offset = pointer_offset(sector_pointers[1])?;
    if first_member_offset <= route_table_offset || first_member_offset > bytes.len() {
        return Err("路由表范围非法".to_string());
    }
    let route_table_size = first_member_offset - route_table_offset;
    if route_table_size % 2 != 0 {
        return Err("路由表长度不是偶数".to_string());
    }

    let mut routing_entries = Vec::new();
    let mut route_terminator_offset = None;
    for offset in (route_table_offset..first_member_offset).step_by(2) {
        let pair = [bytes[offset], bytes[offset + 1]];
        if pair == ROUTE_TERMINATOR {
            route_terminator_offset = Some(offset);
            break;
        }
        let archive_id = pair[0];
        let local_member = pair[1];
        if !(1..=25).contains(&archive_id) {
            return Err(format!(
                "路由项 {} 的 archive_id 非法: {} (offset=0x{offset:X})",
                routing_entries.len() + 1,
                archive_id
            ));
        }
        if local_member == 0 {
            return Err(format!(
                "路由项 {} 的 local_member 为 0 (offset=0x{offset:X})",
                routing_entries.len() + 1
            ));
        }
        routing_entries.push(RoutingEntry {
            logical_id: routing_entries.len() + 1,
            archive_id,
            archive_letter: char::from(b'@' + archive_id),
            local_member,
        });
    }
    let route_terminator_offset =
        route_terminator_offset.ok_or_else(|| "路由表范围内没有找到终止项 1A 00".to_string())?;

    let mut members = Vec::with_capacity(sector_pointers.len() - 2);
    for local_member in 1..(sector_pointers.len() - 1) {
        let start_sector = sector_pointers[local_member];
        let end_sector = sector_pointers[local_member + 1];
        let offset = pointer_offset(start_sector)?;
        let end = pointer_offset(end_sector)?;
        if end > bytes.len() || offset >= end {
            return Err(format!(
                "本地成员 {local_member} 范围非法: 0x{offset:X}..0x{end:X}"
            ));
        }
        let file = format!("members/member_{local_member:03}.bin");
        members.push(MemberInfo {
            local_member,
            start_sector,
            end_sector,
            offset,
            size: end - offset,
            sha256: sha256_hex(&bytes[offset..end]),
            file,
        });
    }

    let covered = SECTOR_SIZE
        .checked_add(route_table_size)
        .and_then(|value| value.checked_add(members.iter().map(|item| item.size).sum()))
        .ok_or_else(|| "计算覆盖字节数时溢出".to_string())?;
    if covered != bytes.len() {
        return Err(format!(
            "内部错误: header+routing+members 覆盖 {covered} 字节，文件为 {} 字节",
            bytes.len()
        ));
    }

    Ok(ParsedArchive {
        sector_pointers,
        routing_entries,
        route_table_offset,
        route_table_size,
        route_terminator_offset,
        members,
    })
}

pub fn rebuild_archive(source: &[u8], member_payloads: &[Vec<u8>]) -> Result<PackReport> {
    let parsed = parse_archive(source)?;
    if member_payloads.len() != parsed.members.len() {
        return Err(format!(
            "成员数量不匹配: 输入包={}，回封数据={}",
            parsed.members.len(),
            member_payloads.len()
        ));
    }

    let mut changed_members = 0usize;
    for (member, payload) in parsed.members.iter().zip(member_payloads) {
        let original = &source[member.offset..member.offset + member.size];
        if payload.as_slice() != original {
            changed_members += 1;
        }
    }
    if changed_members == 0 {
        return Ok(PackReport {
            bytes: source.to_vec(),
            members: member_payloads.len(),
            changed_members,
            source_bytes: source.len(),
            output_bytes: source.len(),
        });
    }

    let route_start = parsed.route_table_offset;
    let route_end = route_start
        .checked_add(parsed.route_table_size)
        .ok_or_else(|| "路由区范围溢出".to_string())?;
    let mut rebuilt = Vec::new();
    rebuilt.extend_from_slice(&source[..SECTOR_SIZE]);
    rebuilt.extend_from_slice(&source[route_start..route_end]);

    let mut pointers = Vec::with_capacity(member_payloads.len() + 2);
    pointers.push(parsed.sector_pointers[0]);
    pointers.push(parsed.sector_pointers[1]);
    let mut next_sector = parsed.sector_pointers[1];
    for payload in member_payloads {
        let member_start = rebuilt.len();
        rebuilt.extend_from_slice(payload);
        let sectors = payload.len().div_ceil(SECTOR_SIZE).max(1);
        let padded_size = sectors
            .checked_mul(SECTOR_SIZE)
            .ok_or_else(|| "成员补齐长度溢出".to_string())?;
        rebuilt.resize(
            member_start
                .checked_add(padded_size)
                .ok_or_else(|| "DAT 输出长度溢出".to_string())?,
            0,
        );
        next_sector = next_sector
            .checked_add(
                u16::try_from(sectors).map_err(|_| "单个成员超过 DAT 扇区范围".to_string())?,
            )
            .ok_or_else(|| "DAT 扇区指针超过 u16".to_string())?;
        pointers.push(next_sector);
    }

    if rebuilt.len() != pointer_offset(next_sector)? {
        return Err(format!(
            "DAT 重建长度与尾指针不一致: bytes=0x{:X}, pointer=0x{:X}",
            rebuilt.len(),
            pointer_offset(next_sector)?
        ));
    }
    for (index, pointer) in pointers.iter().enumerate() {
        let offset = index * 2;
        rebuilt[offset..offset + 2].copy_from_slice(&pointer.to_le_bytes());
    }
    parse_archive(&rebuilt).map_err(|error| format!("重建 DAT 结构复核失败: {error}"))?;
    let output_bytes = rebuilt.len();
    Ok(PackReport {
        bytes: rebuilt,
        members: member_payloads.len(),
        changed_members,
        source_bytes: source.len(),
        output_bytes,
    })
}

pub fn inspect_archive(path: &Path) -> Result<(usize, usize, usize)> {
    let bytes = fs::read(path).map_err(|error| format!("无法读取 {}: {error}", path.display()))?;
    let parsed = parse_archive(&bytes).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok((
        parsed.members.len(),
        parsed.routing_entries.len(),
        parsed.members.iter().map(|item| item.size).sum(),
    ))
}

pub fn preview_mappings(inputs: &[PathBuf]) -> Result<Vec<(PathBuf, String)>> {
    if inputs.is_empty() {
        return Err("至少需要一个 DAT 输入文件".to_string());
    }
    let mut seen = HashSet::new();
    let mut mappings = Vec::with_capacity(inputs.len());
    for input in inputs {
        let stem = input
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", input.display()))?;
        validate_output_segment(stem)?;
        let folded = stem.to_uppercase();
        if !seen.insert(folded) {
            return Err(format!("多个输入会映射到同一输出目录: {stem}"));
        }
        mappings.push((input.clone(), stem.to_string()));
    }
    Ok(mappings)
}

pub fn unpack_batch(
    inputs: &[PathBuf],
    output_root: &Path,
    overwrite: bool,
) -> Result<UnpackReport> {
    let mappings = preview_mappings(inputs)?;
    validate_output_target(output_root, overwrite)?;

    let mut prepared = Vec::with_capacity(mappings.len());
    for (input, output_dir) in mappings {
        let metadata = fs::metadata(&input)
            .map_err(|error| format!("无法读取输入元数据 {}: {error}", input.display()))?;
        if !metadata.is_file() {
            return Err(format!("输入不是普通文件: {}", input.display()));
        }
        let bytes =
            fs::read(&input).map_err(|error| format!("无法读取 {}: {error}", input.display()))?;
        let parsed = parse_archive(&bytes)
            .map_err(|error| format!("解析 {} 失败: {error}", input.display()))?;
        let source_file = input
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("输入文件名无法表示为 Unicode: {}", input.display()))?
            .to_string();
        let manifest = ArchiveManifest {
            source_file,
            source_bytes: bytes.len(),
            source_sha256: sha256_hex(&bytes),
            output_dir: output_dir.clone(),
            sector_size: SECTOR_SIZE,
            sector_pointers: parsed.sector_pointers.clone(),
            route_table_offset: parsed.route_table_offset,
            route_table_size: parsed.route_table_size,
            route_terminator_offset: parsed.route_terminator_offset,
            route_table_file: "routing.bin".to_string(),
            header_file: "header.bin".to_string(),
            routing_entries: parsed.routing_entries.clone(),
            members: parsed.members.clone(),
        };
        prepared.push(PreparedArchive {
            bytes,
            output_dir,
            manifest,
        });
    }

    let archive_roots = prepared
        .iter()
        .map(|item| item.output_dir.clone())
        .collect::<Vec<_>>();
    let workspace = WorkspaceManifest {
        _format: WORKSPACE_FORMAT.to_string(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        role_paths: RolePaths { archive_roots },
        archives: prepared.iter().map(|item| item.manifest.clone()).collect(),
    };

    write_transactional(output_root, overwrite, &workspace, &prepared)?;

    Ok(UnpackReport {
        archives: prepared.len(),
        members: prepared
            .iter()
            .map(|item| item.manifest.members.len())
            .sum(),
        member_bytes: prepared
            .iter()
            .flat_map(|item| item.manifest.members.iter())
            .map(|member| member.size as u64)
            .sum(),
        routing_entries: prepared
            .iter()
            .map(|item| item.manifest.routing_entries.len())
            .sum(),
        output_root: output_root.to_path_buf(),
    })
}

fn write_transactional(
    output_root: &Path,
    overwrite: bool,
    workspace: &WorkspaceManifest,
    prepared: &[PreparedArchive],
) -> Result<()> {
    let parent = output_root.parent().unwrap_or_else(|| Path::new("."));
    let name = output_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输出目录名无法表示为 Unicode: {}", output_root.display()))?;
    let nonce = std::process::id();
    let staging = parent.join(format!(".{name}.staging-{nonce}"));
    let backup = parent.join(format!(".{name}.backup-{nonce}"));
    if staging.exists() || backup.exists() {
        return Err("事务目录已存在，请清理同名 staging/backup 后重试".to_string());
    }

    fs::create_dir_all(&staging)
        .map_err(|error| format!("无法创建事务目录 {}: {error}", staging.display()))?;
    let write_result = (|| -> Result<()> {
        for item in prepared {
            let archive_root = staging.join(&item.output_dir);
            fs::create_dir_all(archive_root.join("members"))
                .map_err(|error| format!("无法创建输出目录 {}: {error}", archive_root.display()))?;
            fs::write(archive_root.join("header.bin"), &item.bytes[..SECTOR_SIZE])
                .map_err(|error| format!("写出 header.bin 失败: {error}"))?;
            let route_start = item.manifest.route_table_offset;
            let route_end = route_start + item.manifest.route_table_size;
            fs::write(
                archive_root.join("routing.bin"),
                &item.bytes[route_start..route_end],
            )
            .map_err(|error| format!("写出 routing.bin 失败: {error}"))?;
            for member in &item.manifest.members {
                let end = member.offset + member.size;
                fs::write(
                    archive_root.join(&member.file),
                    &item.bytes[member.offset..end],
                )
                .map_err(|error| {
                    format!("写出 {}/{} 失败: {error}", item.output_dir, member.file)
                })?;
            }
        }
        let mut json = serde_json::to_string_pretty(workspace)
            .map_err(|error| format!("序列化 workspace.json 失败: {error}"))?;
        json.push('\n');
        fs::write(staging.join("workspace.json"), json.as_bytes())
            .map_err(|error| format!("写出 workspace.json 失败: {error}"))?;
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    if output_root.exists() {
        if !overwrite {
            let _ = fs::remove_dir_all(&staging);
            return Err(format!(
                "输出目录已存在，拒绝覆盖: {}",
                output_root.display()
            ));
        }
        fs::rename(output_root, &backup).map_err(|error| {
            let _ = fs::remove_dir_all(&staging);
            format!("无法把旧输出移入事务备份: {error}")
        })?;
        if let Err(error) = fs::rename(&staging, output_root) {
            let _ = fs::rename(&backup, output_root);
            let _ = fs::remove_dir_all(&staging);
            return Err(format!("无法提交新输出，已尝试恢复旧输出: {error}"));
        }
        fs::remove_dir_all(&backup)
            .map_err(|error| format!("新输出已提交，但无法删除事务备份: {error}"))?;
    } else {
        fs::rename(&staging, output_root).map_err(|error| format!("无法提交输出目录: {error}"))?;
    }
    Ok(())
}

fn validate_output_target(output_root: &Path, overwrite: bool) -> Result<()> {
    if !output_root.exists() {
        return Ok(());
    }
    if !overwrite {
        return Err(format!(
            "输出目录已存在，拒绝覆盖: {}",
            output_root.display()
        ));
    }
    if !output_root.is_dir() {
        return Err(format!("输出目标不是目录: {}", output_root.display()));
    }
    let manifest_path = output_root.join("workspace.json");
    let text = fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "--overwrite 只允许覆盖本工具工作区；无法读取 {}: {error}",
            manifest_path.display()
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| format!("旧 workspace.json 无效，拒绝覆盖: {error}"))?;
    if value.get("_format").and_then(|item| item.as_str()) != Some(WORKSPACE_FORMAT) {
        return Err("旧 workspace.json 不是本工具格式，拒绝覆盖".to_string());
    }
    Ok(())
}

fn validate_output_segment(value: &str) -> Result<()> {
    if value.is_empty() || value == "." || value == ".." {
        return Err(format!("不安全的输出目录名: {value:?}"));
    }
    if value
        .chars()
        .any(|ch| ch < ' ' || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'))
        || value.ends_with([' ', '.'])
    {
        return Err(format!("Windows 下不安全的输出目录名: {value:?}"));
    }
    let base = value
        .split('.')
        .next()
        .unwrap_or(value)
        .to_ascii_uppercase();
    let reserved = matches!(
        base.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    );
    if reserved {
        return Err(format!("Windows 保留设备名不能作为输出目录: {value:?}"));
    }
    Ok(())
}

fn pointer_offset(pointer: u16) -> Result<usize> {
    usize::from(pointer)
        .checked_sub(1)
        .and_then(|value| value.checked_mul(SECTOR_SIZE))
        .ok_or_else(|| format!("扇区指针换算溢出: {pointer}"))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let pair = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("读取 u16 越界: offset=0x{offset:X}"))?;
    Ok(u16::from_le_bytes([pair[0], pair[1]]))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02X}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_archive() -> Vec<u8> {
        let mut bytes = vec![0u8; 0x500];
        for (index, pointer) in [2u16, 3, 5, 6].iter().enumerate() {
            bytes[index * 2..index * 2 + 2].copy_from_slice(&pointer.to_le_bytes());
        }
        bytes[0x100..0x108].copy_from_slice(&[1, 1, 1, 2, 2, 1, 0x1A, 0]);
        bytes[0x200..0x400].fill(0x41);
        bytes[0x400..0x500].fill(0x42);
        bytes
    }

    #[test]
    fn parses_members_and_routes() {
        let bytes = sample_archive();
        let parsed = parse_archive(&bytes).expect("parse");
        assert_eq!(parsed.sector_pointers, vec![2, 3, 5, 6]);
        assert_eq!(parsed.route_table_offset, 0x100);
        assert_eq!(parsed.route_table_size, 0x100);
        assert_eq!(parsed.routing_entries.len(), 3);
        assert_eq!(parsed.members.len(), 2);
        assert_eq!(parsed.members[0].offset, 0x200);
        assert_eq!(parsed.members[0].size, 0x200);
        assert_eq!(parsed.members[1].offset, 0x400);
        assert_eq!(parsed.members[1].size, 0x100);
    }

    #[test]
    fn rejects_unaligned_file() {
        let mut bytes = sample_archive();
        bytes.pop();
        assert!(parse_archive(&bytes).unwrap_err().contains("0x100 对齐"));
    }

    #[test]
    fn rejects_missing_route_terminator() {
        let mut bytes = sample_archive();
        for pair in bytes[0x100..0x200].chunks_exact_mut(2) {
            pair.copy_from_slice(&[1, 1]);
        }
        assert!(parse_archive(&bytes).unwrap_err().contains("1A 00"));
    }

    #[test]
    fn rejects_non_monotonic_index() {
        let mut bytes = sample_archive();
        bytes[4..6].copy_from_slice(&3u16.to_le_bytes());
        assert!(parse_archive(&bytes).unwrap_err().contains("不递增"));
    }

    #[test]
    fn unchanged_rebuild_is_byte_exact() {
        let source = sample_archive();
        let parsed = parse_archive(&source).unwrap();
        let members = parsed
            .members
            .iter()
            .map(|member| source[member.offset..member.offset + member.size].to_vec())
            .collect::<Vec<_>>();
        let report = rebuild_archive(&source, &members).unwrap();
        assert_eq!(report.bytes, source);
        assert_eq!(report.changed_members, 0);
    }

    #[test]
    fn growing_member_updates_following_pointer() {
        let source = sample_archive();
        let parsed = parse_archive(&source).unwrap();
        let mut members = parsed
            .members
            .iter()
            .map(|member| source[member.offset..member.offset + member.size].to_vec())
            .collect::<Vec<_>>();
        members[0].extend_from_slice(&vec![0x43; SECTOR_SIZE]);
        let report = rebuild_archive(&source, &members).unwrap();
        let rebuilt = parse_archive(&report.bytes).unwrap();
        assert_eq!(rebuilt.sector_pointers, vec![2, 3, 6, 7]);
        assert_eq!(rebuilt.members[0].size, 0x300);
        assert_eq!(rebuilt.members[1].offset, 0x500);
    }
}
