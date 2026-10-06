use fivec_new::{sha256, Archive};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let path = std::env::temp_dir().join(format!(
                "fivec-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("{e}"),
            }
        }
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn fat(bps: usize, value: u8) -> Vec<u8> {
    let mut raw = vec![0; 32 * bps];
    raw[11..13].copy_from_slice(&(bps as u16).to_le_bytes());
    raw[13] = 1;
    raw[14..16].copy_from_slice(&1u16.to_le_bytes());
    raw[16] = 2;
    raw[17..19].copy_from_slice(&16u16.to_le_bytes());
    raw[19..21].copy_from_slice(&32u16.to_le_bytes());
    raw[21] = 0xf8;
    raw[22..24].copy_from_slice(&1u16.to_le_bytes());
    for offset in [bps, bps * 2] {
        raw[offset..offset + 5].copy_from_slice(&[0xf8, 0xff, 0xff, 0xff, 0x0f]);
    }
    let root = bps * 3;
    raw[root..root + 11].copy_from_slice(b"HELLO   DAT");
    raw[root + 11] = 0x20;
    raw[root + 26..root + 28].copy_from_slice(&2u16.to_le_bytes());
    raw[root + 28..root + 32].copy_from_slice(&100u32.to_le_bytes());
    raw[bps * 4..bps * 4 + 100].fill(value);
    raw
}

fn anex(raw: &[u8], cylinders: u32, heads: u32, sectors: u32, bps: u32) -> Vec<u8> {
    let mut source = vec![0; 32];
    for (i, field) in [
        0u32,
        0,
        32,
        raw.len() as u32,
        bps,
        sectors,
        heads,
        cylinders,
    ]
    .into_iter()
    .enumerate()
    {
        source[i * 4..i * 4 + 4].copy_from_slice(&field.to_le_bytes());
    }
    source.extend_from_slice(raw);
    source
}

fn floppy() -> Vec<u8> {
    anex(&fat(512, 0x5a), 4, 1, 8, 512)
}

fn hdi() -> Vec<u8> {
    let mut raw = vec![0; 24 * 2 * 8 * 512];
    for (index, start) in [1usize, 8].into_iter().enumerate() {
        let entry = 512 + index * 32;
        raw[entry..entry + 2].copy_from_slice(&[0xa1, 0x81]);
        raw[entry + 10..entry + 12].copy_from_slice(&(start as u16).to_le_bytes());
        raw[entry + 14..entry + 16].copy_from_slice(&((start + 3) as u16).to_le_bytes());
        let base = start * 2 * 8 * 512;
        raw[base..base + 32 * 1024].copy_from_slice(&fat(1024, index as u8 + 1));
    }
    anex(&raw, 24, 2, 8, 512)
}

fn d88(raw: &[u8]) -> Vec<u8> {
    let mut source = vec![0; 0x2b0];
    for head in 0..2usize {
        let start = source.len() as u32;
        source[0x20 + head * 4..0x24 + head * 4].copy_from_slice(&start.to_le_bytes());
        for sector in (1..=16usize).rev() {
            let mut record = [0; 16];
            record[..6].copy_from_slice(&[0, head as u8, sector as u8, 2, 16, 0]);
            record[14..16].copy_from_slice(&512u16.to_le_bytes());
            source.extend_from_slice(&record);
            let base = (head * 16 + sector - 1) * 512;
            source.extend_from_slice(&raw[base..base + 512]);
        }
    }
    let size = source.len() as u32;
    source[28..32].copy_from_slice(&size.to_le_bytes());
    source
}

#[test]
fn partition_units_bounds_and_multiple_volumes_are_independent() {
    let image = Archive::from_bytes("renamed.bin", hdi()).unwrap();
    assert_eq!(image.volumes().len(), 2);
    for (i, volume) in image.volumes().iter().enumerate() {
        assert_eq!(volume.logical_byte_length, 32 * 1024);
        let file = &volume.filesystem.as_ref().unwrap().files[0];
        assert_eq!(
            image.read_file(&volume.id, &file.id).unwrap(),
            vec![i as u8 + 1; 100]
        );
        assert_eq!(
            file.source_ranges[0].source_offset,
            32 + volume.logical_byte_offset + 4096
        );
    }
    assert_eq!(image.selected_volumes("all").unwrap().len(), 2);
    assert!(image.selected_volumes("no-such-volume").is_err());
}

#[test]
fn partition_overrun_overlap_and_unverified_end_never_extract() {
    for (offset, value) in [
        (32 + 512 + 14, 1u16),
        (32 + 512 + 14, 8),
        (32 + 512 + 12, 1),
    ] {
        let mut data = hdi();
        data[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        let image = Archive::from_bytes("bad.hdi", data).unwrap();
        assert!(image.volumes()[0].filesystem.is_none());
        assert!(image.selected_volumes("all").is_err());
        assert!(!image.volumes()[0].diagnostics.is_empty());
        if offset == 32 + 512 + 14 && value == 8 {
            assert!(image.volumes()[1].filesystem.is_none());
        }
    }
}

#[test]
fn concatenated_interleaved_d88_maps_each_filesystem_to_its_own_source() {
    let mut source = d88(&fat(512, 1));
    let second_start = source.len();
    source.extend(d88(&fat(512, 2)));
    let image = Archive::from_bytes("joined.d88", source).unwrap();
    assert_eq!(image.volumes().len(), 2);
    for (index, volume) in image.volumes().iter().enumerate() {
        let entry = &volume.filesystem.as_ref().unwrap().files[0];
        assert_eq!(
            image.read_file(&volume.id, &entry.id).unwrap(),
            vec![index as u8 + 1; 100]
        );
        if index == 1 {
            assert!(entry.source_ranges[0].source_offset > second_start);
        }
    }
}

#[test]
fn corrupted_fat_is_inspectable_but_never_empty_success() {
    let mut source = floppy();
    source[32 + 512 + 3] = 2;
    source[32 + 512 + 4] = 0;
    let image = Archive::from_bytes("bad.fdi", source).unwrap();
    assert!(image.volumes()[0].filesystem.is_none());
    assert!(image.volumes()[0].diagnostics[0].contains("invalid FAT12"));
    assert!(image.selected_volumes("all").is_err());
}

#[test]
fn export_is_read_only_during_prepare_and_preserves_hashes() {
    let temp = Temp::new();
    let source = temp.0.join("来源 空格 [x] $().fdi");
    fs::write(&source, floppy()).unwrap();
    let image = Archive::open(&source).unwrap();
    let output = temp.0.join("结果 [x] $()");
    let job = image.prepare_export("all", &output, false).unwrap();
    assert!(!output.exists());
    let report = job.execute().unwrap();
    assert_eq!((report.files, report.bytes), (1, 100));
    let path = output.join("disk-000-whole/HELLO.DAT");
    assert_eq!(fs::read(path).unwrap(), vec![0x5a; 100]);
    assert_eq!(
        sha256(&fs::read(source).unwrap()),
        image.inspection().source_sha256
    );
    assert!(image.prepare_export("all", &output, false).is_err());
    let replaced = image
        .prepare_export("all", &output, true)
        .unwrap()
        .execute()
        .unwrap();
    assert!(replaced
        .warnings
        .iter()
        .any(|s| s.contains("retained for recovery")));
    let backup = fs::read_dir(&temp.0)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".fivec-backup")
        })
        .unwrap();
    assert_eq!(
        fs::read(backup.join("previous/disk-000-whole/HELLO.DAT")).unwrap(),
        vec![0x5a; 100]
    );
}

#[test]
fn output_changes_after_prepare_and_unrelated_files_are_preserved() {
    let temp = Temp::new();
    let image = Archive::from_bytes("test.fdi", floppy()).unwrap();
    let output = temp.0.join("out");
    let job = image.prepare_export("all", &output, false).unwrap();
    fs::create_dir(&output).unwrap();
    fs::write(output.join("unrelated.txt"), b"keep").unwrap();
    assert!(job.execute().is_err());
    assert_eq!(fs::read(output.join("unrelated.txt")).unwrap(), b"keep");
    assert!(image.prepare_export("all", &output, true).is_err());
    let owned = temp.0.join("owned");
    image
        .prepare_export("all", &owned, false)
        .unwrap()
        .execute()
        .unwrap();
    let job = image.prepare_export("all", &owned, true).unwrap();
    fs::write(owned.join("disk-000-whole/HELLO.DAT"), b"edited").unwrap();
    assert!(job.execute().is_err());
    assert_eq!(
        fs::read(owned.join("disk-000-whole/HELLO.DAT")).unwrap(),
        b"edited"
    );
    assert!(fs::read_dir(&temp.0).unwrap().all(|e| !e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".fivec-stage")));
}

#[test]
fn changed_source_and_source_containing_output_are_rejected() {
    let temp = Temp::new();
    let source = temp.0.join("input.fdi");
    fs::write(&source, floppy()).unwrap();
    let image = Archive::open(&source).unwrap();
    assert!(image.prepare_export("all", &temp.0, true).is_err());
    assert!(image.prepare_export("all", &source, true).is_err());
    #[cfg(windows)]
    assert!(image
        .prepare_export("all", temp.0.to_string_lossy().to_uppercase(), true)
        .is_err());
    let output = temp.0.join("out");
    let job = image.prepare_export("all", &output, false).unwrap();
    fs::write(&source, b"changed").unwrap();
    assert!(job.execute().is_err());
    assert!(!output.exists());
}

#[test]
fn explicit_volume_exports_only_selected_partition() {
    let temp = Temp::new();
    let image = Archive::from_bytes("test.hdi", hdi()).unwrap();
    let id = &image.volumes()[1].id;
    let job = image.prepare_export(id, temp.0.join("out"), false).unwrap();
    assert_eq!(job.manifest().files.len(), 1);
    assert_eq!(job.manifest().selected_volumes, std::slice::from_ref(id));
    assert_eq!(job.execute().unwrap().files, 1);
}

#[test]
fn cli_help_complete_commands_eof_and_path_prefill_use_shared_panel() {
    let temp = Temp::new();
    let source = temp.0.join("磁盘 ' [a] $().fdi");
    fs::write(&source, floppy()).unwrap();
    let binary = env!("CARGO_BIN_EXE_fivec_new");
    let help = Command::new(binary).arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("extract"));
    let output = temp.0.join("输出 [b] $()");
    let extract = || {
        Command::new(binary)
            .arg("extract")
            .arg("--source")
            .arg(&source)
            .arg("--output")
            .arg(&output)
            .stdin(Stdio::null())
            .output()
            .unwrap()
    };
    assert!(extract().status.success());
    assert!(!extract().status.success());
    let eof = Command::new(binary).stdin(Stdio::null()).output().unwrap();
    assert!(eof.status.success());
    let mut child = Command::new(binary)
        .arg(&source)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"1\nC\n0\n1\n0\n0\n")
        .unwrap();
    let panel = child.wait_with_output().unwrap();
    assert!(panel.status.success());
    let text = String::from_utf8_lossy(&panel.stdout);
    assert!(text.contains("--source"));
    assert!(
        text.matches("磁盘 '").count() >= 2,
        "parameters/prefill were not retained: {text}"
    );
    assert_eq!(sha256(&fs::read(&source).unwrap()), sha256(&floppy()));
}
