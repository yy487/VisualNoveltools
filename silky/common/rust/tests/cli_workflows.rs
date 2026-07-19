use silky_common::archive::MANIFEST_NAME;
use silky_common::codec::encode_text;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(label: &str) -> Self {
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "silky Rust & 中文 {label}-{}-{nonce}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn mes_fixture() -> Vec<u8> {
    let body = encode_text("本文", "cp932").unwrap();
    let mut code = Vec::new();
    code.push(0x19);
    code.extend_from_slice(&0u32.to_be_bytes());
    code.push(0x0b);
    code.extend_from_slice(&body);
    code.push(0);
    code.push(0x00);
    let mut file = Vec::new();
    file.extend_from_slice(&1u32.to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    file.extend_from_slice(&code);
    file
}

fn run(executable: &str, args: &[&Path]) -> bool {
    let mut command = Command::new(executable);
    for arg in args {
        command.arg(arg);
    }
    command.status().unwrap().success()
}

#[test]
fn extract_and_inject_exes_roundtrip_unicode_paths() {
    let root = TestRoot::new("mes");
    let mes = root.0.join("场景 01.MES");
    let json = root.0.join("场景 01.json");
    let injected = root.0.join("场景 01 injected.MES");
    let source = mes_fixture();
    fs::write(&mes, &source).unwrap();

    assert!(run(env!("CARGO_BIN_EXE_extract"), &[&mes, &json]));
    assert!(run(env!("CARGO_BIN_EXE_inject"), &[&mes, &json, &injected]));
    assert_eq!(fs::read(&injected).unwrap(), source);

    // Existing output must be rejected without changing it.
    let before = fs::read(&json).unwrap();
    assert!(!run(env!("CARGO_BIN_EXE_extract"), &[&mes, &json]));
    assert_eq!(fs::read(&json).unwrap(), before);
}

#[test]
fn repack_and_unpack_exes_roundtrip_tree() {
    let root = TestRoot::new("arc");
    let input = root.0.join("输入 文件");
    let nested = input.join("サブ");
    fs::create_dir_all(&nested).unwrap();
    fs::write(input.join("A.MES"), b"abcabcabcabcabc").unwrap();
    fs::write(nested.join("B.bin"), (0..=255).collect::<Vec<u8>>()).unwrap();
    let archive = root.0.join("Script 测试.arc");
    let unpacked = root.0.join("解包 输出");

    assert!(run(env!("CARGO_BIN_EXE_repack"), &[&input, &archive]));
    assert!(run(env!("CARGO_BIN_EXE_unpack"), &[&archive, &unpacked]));
    assert_eq!(
        fs::read(unpacked.join("A.MES")).unwrap(),
        b"abcabcabcabcabc"
    );
    assert_eq!(
        fs::read(unpacked.join("サブ").join("B.bin")).unwrap(),
        (0..=255).collect::<Vec<u8>>()
    );
    assert!(unpacked.join(MANIFEST_NAME).is_file());
}
