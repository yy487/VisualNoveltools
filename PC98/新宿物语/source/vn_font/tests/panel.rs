use std::ffi::OsStr;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::{Operation, Parameters, Progress, Value};
use vn_font::{font_88, font_98, panel::*};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "vn-font-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(self.0.starts_with(std::env::temp_dir()));
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn request(backend: &str) -> Vec<u8> {
    serde_json::to_vec(&FontRequest {
        schema_version: 1,
        backend: backend.into(),
        final_texts: vec!["你好日本語".into()],
        reserved_cp932: Vec::new(),
    })
    .unwrap()
}
fn setup(f: &Fixture, pc88: bool) -> Parameters {
    fs::write(
        f.path("原始 & 字库"),
        if pc88 {
            vec![0xa5; font_88::KANJI1_ROM_SIZE]
        } else {
            font_98::EMBEDDED_FONT.to_vec()
        },
    )
    .unwrap();
    fs::write(
        f.path("完整构建请求.json"),
        request(if pc88 { "pc88-kanji1" } else { "pc98-np2" }),
    )
    .unwrap();
    let mut params = Parameters::default();
    params.set("source", Value::Path(f.path("原始 & 字库")));
    params.set("request", Value::Path(f.path("完整构建请求.json")));
    params.set("output", Value::Path(f.path("独立 成品")));
    params.set("face", Value::Text(font_98::FONT_FACE.into()));
    params
}
struct Silent;
impl Progress for Silent {
    fn report(&mut self, _: &str) -> vn_cli::Result<()> {
        Ok(())
    }
}

#[test]
fn preparation_writes_nothing_and_commits_all_files_together() {
    let f = Fixture::new();
    let params = setup(&f, true);
    let before = fs::read(params.path("source").unwrap()).unwrap();
    let job = FontOperation::pc88().prepare(&params).unwrap();
    assert!(!f.path("独立 成品").exists());
    job.execute(&mut Silent).unwrap();
    assert!(f.path("独立 成品/KANJI1.ROM").is_file());
    assert!(f.path("独立 成品/font_plan.json").is_file());
    assert!(f.path("独立 成品/font_manifest.json").is_file());
    assert_eq!(fs::read(params.path("source").unwrap()).unwrap(), before);
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 3);
}

#[test]
fn existing_and_changed_outputs_are_preserved_on_failure() {
    let f = Fixture::new();
    let mut params = setup(&f, true);
    let job = FontOperation::pc88().prepare(&params).unwrap();
    fs::create_dir(f.path("独立 成品")).unwrap();
    fs::write(f.path("独立 成品/old"), b"old").unwrap();
    assert!(job.execute(&mut Silent).is_err());
    assert_eq!(fs::read(f.path("独立 成品/old")).unwrap(), b"old");
    assert!(FontOperation::pc88().prepare(&params).is_err());
    params.set("overwrite", Value::Flag(true));
    let job = FontOperation::pc88().prepare(&params).unwrap();
    fs::write(f.path("独立 成品/old"), b"changed").unwrap();
    assert!(job.execute(&mut Silent).is_err());
    assert_eq!(fs::read(f.path("独立 成品/old")).unwrap(), b"changed");
    FontOperation::pc88()
        .prepare(&params)
        .unwrap()
        .execute(&mut Silent)
        .unwrap();
    assert!(!f.path("独立 成品/old").exists());
    assert!(f.path("独立 成品/KANJI1.ROM").exists());
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 3);
}

#[test]
fn malformed_requests_never_create_output() {
    let f = Fixture::new();
    let params = setup(&f, true);
    for invalid in [b"{}".to_vec(), request("pc98-np2"), b"[]".to_vec()] {
        fs::write(params.path("request").unwrap(), invalid).unwrap();
        assert!(FontOperation::pc88().prepare(&params).is_err());
        assert!(!f.path("独立 成品").exists());
    }
}

#[test]
fn real_executable_uses_shared_panel_and_noninteractive_commands() {
    let f = Fixture::new();
    let params = setup(&f, true);
    let run = |args: &[&OsStr]| {
        Command::new(env!("CARGO_BIN_EXE_vn_font_tool"))
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap()
    };
    let help = run(&[OsStr::new("--help")]);
    assert!(help.status.success());
    assert!(String::from_utf8(help.stdout).unwrap().contains("build-88"));
    let menu = run(&[]);
    assert!(menu.status.success());
    assert!(String::from_utf8(menu.stdout).unwrap().contains("主菜单"));
    let args = [
        OsStr::new("build-88"),
        OsStr::new("--source"),
        params.path("source").unwrap().as_os_str(),
        OsStr::new("--request"),
        params.path("request").unwrap().as_os_str(),
        OsStr::new("--output"),
        params.path("output").unwrap().as_os_str(),
    ];
    let built = run(&args);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(!run(&args).status.success());
    assert!(!run(&[OsStr::new("build-88")]).status.success());
}

#[test]
fn interactive_prefill_cancel_and_confirm_keep_source_unchanged() {
    let f = Fixture::new();
    let params = setup(&f, true);
    let mut panel = vn_cli::Panel::new("font", "字库", "1", operations()).unwrap();
    let args = [
        params.path("source").unwrap().as_os_str().to_owned(),
        params.path("request").unwrap().as_os_str().to_owned(),
    ];
    let input = format!(
        "1\n3\n{}\np\nr\n0\nr\ny\nq\n",
        params.path("output").unwrap().display()
    );
    let mut output = Vec::new();
    panel
        .run(&args, &mut Cursor::new(input), &mut output)
        .unwrap();
    let log = String::from_utf8(output).unwrap();
    assert!(log.contains("已取消，参数保留"));
    assert!(log.contains("[完成]"));
    assert_eq!(
        fs::read(params.path("source").unwrap()).unwrap(),
        vec![0xa5; font_88::KANJI1_ROM_SIZE]
    );
}

#[cfg(windows)]
#[test]
fn pc98_panel_builds_from_explicit_source_and_preserves_request() {
    let f = Fixture::new();
    let params = setup(&f, false);
    let before = fs::read(params.path("request").unwrap()).unwrap();
    FontOperation::pc98()
        .prepare(&params)
        .unwrap()
        .execute(&mut Silent)
        .unwrap();
    assert!(f.path("独立 成品/font.tmp").is_file());
    assert_eq!(
        fs::read(params.path("source").unwrap()).unwrap(),
        font_98::EMBEDDED_FONT
    );
    assert_eq!(fs::read(params.path("request").unwrap()).unwrap(), before);
}
