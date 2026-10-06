use std::cell::{Cell, RefCell};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, BufRead, Cursor, Read};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use vn_cli::*;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "vn-cli-test-{}-{}",
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
#[derive(Default)]
struct State {
    prepared: RefCell<Vec<Parameters>>,
    executions: RefCell<Vec<String>>,
    prefills: Cell<usize>,
    fail_execute: Cell<bool>,
}
struct Probe {
    state: Rc<State>,
    spec: OperationSpec,
    preview: Preview,
}
impl Operation for Probe {
    fn spec(&self) -> OperationSpec {
        self.spec.clone()
    }
    fn prefill(&self, paths: &[PathBuf], p: &mut Parameters) -> Result<()> {
        self.state.prefills.set(self.state.prefills.get() + 1);
        if !paths.is_empty() {
            p.set("label", Value::Text("预填".into()));
        }
        Ok(())
    }
    fn prepare(&self, p: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        self.state.prepared.borrow_mut().push(p.clone());
        if p.text("label")? == "拒绝" {
            return Err("测试预检失败".into());
        }
        Ok(Box::new(Job {
            state: self.state.clone(),
            preview: self.preview.clone(),
            label: p.text("label")?.into(),
        }))
    }
}
struct Job {
    state: Rc<State>,
    preview: Preview,
    label: String,
}
impl PreparedOperation for Job {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(self: Box<Self>, _: &mut dyn Progress) -> Result<RunReport> {
        self.state.executions.borrow_mut().push(self.label.clone());
        if self.state.fail_execute.replace(false) {
            return Err("测试业务失败".into());
        }
        Ok(RunReport {
            summary: self.label.clone(),
            ..RunReport::default()
        })
    }
}
fn spec(writes: bool) -> OperationSpec {
    let mut s = OperationSpec::new(
        "probe",
        "测试操作",
        vec![
            Field::new("label", "名称", FieldKind::Text)
                .required()
                .default(Value::Text("初值".into())),
            Field::new("translation", "译文", FieldKind::Paths),
            Field::new(
                "mode",
                "模式",
                FieldKind::Choice(vec!["普通".into(), "字库".into()]),
            )
            .default(Value::Text("普通".into())),
        ],
    );
    s.writes = writes;
    s
}
fn panel(writes: bool, preview: Preview) -> (Panel, Rc<State>) {
    let state = Rc::new(State::default());
    let panel = Panel::new(
        "tool.exe",
        "工具",
        "0",
        vec![Box::new(Probe {
            state: state.clone(),
            spec: spec(writes),
            preview,
        })],
    )
    .unwrap();
    (panel, state)
}
fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
fn session(panel: &mut Panel, arguments: &[OsString], script: &str) -> String {
    let mut output = Vec::new();
    panel
        .run(arguments, &mut Cursor::new(script), &mut output)
        .unwrap();
    String::from_utf8(output).unwrap()
}
struct NeverRead;
impl Read for NeverRead {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("非交互命令读取了输入")
    }
}
impl BufRead for NeverRead {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        panic!("非交互命令读取了输入")
    }
    fn consume(&mut self, _: usize) {
        panic!("非交互命令读取了输入")
    }
}

#[test]
fn help_and_explicit_commands_never_prompt_even_when_invalid() {
    let (mut p, state) = panel(false, Preview::default());
    for a in [
        args(&["--help"]),
        args(&["probe", "-h"]),
        args(&["probe", "--label", "完成"]),
    ] {
        p.run(&a, &mut NeverRead, &mut Vec::new()).unwrap();
    }
    assert_eq!(&*state.executions.borrow(), &["完成"]);
    for a in [
        args(&["typo"]),
        args(&["probe", "--unknown"]),
        args(&["probe", "--label"]),
        args(&["probe", "--mode", "错误"]),
    ] {
        assert!(p.run(&a, &mut NeverRead, &mut Vec::new()).is_err());
    }
}

#[test]
fn returning_retains_edits_prefills_once_and_execution_prepares_again() {
    let (mut p, state) = panel(false, Preview::default());
    let log = session(
        &mut p,
        &args(&["--", "拖入的文件"]),
        "1\n1\n自定义\np\n0\n1\nr\n1\nr\nq\n",
    );
    assert_eq!(state.prefills.get(), 1);
    assert_eq!(state.prepared.borrow().len(), 3);
    assert!(state
        .prepared
        .borrow()
        .iter()
        .all(|p| p.text("label").unwrap() == "自定义"));
    assert_eq!(&*state.executions.borrow(), &["自定义", "自定义"]);
    assert!(log.contains("预填"));
}

#[test]
fn editing_after_preview_does_not_execute_stale_job() {
    let (mut p, state) = panel(false, Preview::default());
    session(&mut p, &[], "1\np\n1\n新值\nr\nq\n");
    assert_eq!(state.prepared.borrow().len(), 2);
    assert_eq!(&*state.executions.borrow(), &["新值"]);
}

#[test]
fn recoverable_failures_keep_form_and_parameters() {
    let (mut p, state) = panel(false, Preview::default());
    state.fail_execute.set(true);
    let log = session(&mut p, &[], "1\n1\n拒绝\nr\n1\n恢复\nr\nr\nq\n");
    assert!(log.contains("[预检失败]"));
    assert!(log.contains("[操作失败]"));
    assert!(log.contains("[完成] 恢复"));
    assert_eq!(&*state.executions.borrow(), &["恢复", "恢复"]);
}

#[test]
fn eof_at_every_prompt_exits_without_execution_or_partial_list_edits() {
    let f = Fixture::new();
    for script in [
        "",
        "1\n",
        "1\n1\n",
        "1\n2\n部分路径\n",
        "1\nr\n",
        "错误\n",
        "1\n3\n错误选项\n",
    ] {
        let (mut p, state) = panel(
            true,
            Preview {
                outputs: vec![f.path("output")],
                ..Preview::default()
            },
        );
        session(&mut p, &[], script);
        assert!(state.executions.borrow().is_empty(), "script: {script}");
        if script.contains("部分路径") {
            session(&mut p, &[], "1\np\nq\n");
            assert!(state
                .prepared
                .borrow()
                .last()
                .unwrap()
                .paths("translation")
                .unwrap()
                .is_empty());
        }
    }
}

#[test]
fn cancelled_list_edit_preserves_previous_selection() {
    let (mut p, state) = panel(false, Preview::default());
    session(
        &mut p,
        &[],
        "1\n2\n原译文.json\n\n2\n新译文.json\n:cancel\nr\nq\n",
    );
    assert_eq!(
        state.prepared.borrow()[0].paths("translation").unwrap(),
        &[PathBuf::from("原译文.json")]
    );
}

#[test]
fn write_requires_confirmation_and_path_only_does_not_run() {
    let f = Fixture::new();
    let (mut p, state) = panel(
        true,
        Preview {
            outputs: vec![f.path("output")],
            ..Preview::default()
        },
    );
    session(
        &mut p,
        &[f.0.clone().into_os_string()],
        "1\nr\n\nr\n0\nr\ny\nq\n",
    );
    assert_eq!(state.prepared.borrow().len(), 3);
    assert_eq!(state.executions.borrow().len(), 1);
}

#[test]
fn existing_outputs_require_explicit_overwrite() {
    let f = Fixture::new();
    let output = f.path("output");
    fs::write(&output, b"original").unwrap();
    let (mut p, state) = panel(
        true,
        Preview {
            outputs: vec![output],
            ..Preview::default()
        },
    );
    assert!(p
        .run(&args(&["probe"]), &mut NeverRead, &mut Vec::new())
        .is_err());
    assert!(state.executions.borrow().is_empty());
    p.run(
        &args(&["probe", "--overwrite"]),
        &mut NeverRead,
        &mut Vec::new(),
    )
    .unwrap();
    assert_eq!(state.executions.borrow().len(), 1);
}

#[test]
fn output_cannot_overlap_any_input_or_another_output_even_with_overwrite() {
    let f = Fixture::new();
    fs::create_dir(f.path("source")).unwrap();
    fs::create_dir(f.path("translated")).unwrap();
    let cases = [
        (vec![f.path("source")], vec![f.path("source/new")]),
        (vec![f.path("source")], vec![f.0.clone()]),
        (
            vec![f.path("translated")],
            vec![f.path("translated/modified.json")],
        ),
        (vec![f.path("source")], vec![f.path("source/../source/new")]),
        (vec![], vec![f.path("out"), f.path("out/nested")]),
    ];
    for (inputs, outputs) in cases {
        let (mut p, state) = panel(
            true,
            Preview {
                inputs,
                outputs,
                ..Preview::default()
            },
        );
        assert!(p
            .run(
                &args(&["probe", "--overwrite"]),
                &mut NeverRead,
                &mut Vec::new()
            )
            .is_err());
        assert!(state.executions.borrow().is_empty());
    }
}

#[cfg(windows)]
#[test]
fn windows_case_and_junction_aliases_are_protected() {
    let f = Fixture::new();
    let original = f.path("Source");
    fs::create_dir(&original).unwrap();
    let alias = f.path("alias");
    let status = std::process::Command::new("cmd.exe")
        .args([
            OsStr::new("/d"),
            OsStr::new("/c"),
            OsStr::new("mklink"),
            OsStr::new("/J"),
            alias.as_os_str(),
            original.as_os_str(),
        ])
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    for output in [f.path("source/new"), alias.join("new")] {
        let (mut p, state) = panel(
            true,
            Preview {
                inputs: vec![original.clone()],
                outputs: vec![output],
                ..Preview::default()
            },
        );
        assert!(p
            .run(
                &args(&["probe", "--overwrite"]),
                &mut NeverRead,
                &mut Vec::new()
            )
            .is_err());
        assert!(state.executions.borrow().is_empty());
    }
    fs::remove_dir(alias).unwrap();
}

struct OnConfirm {
    input: Cursor<&'static str>,
    output_path: PathBuf,
}
impl Read for OnConfirm {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        self.input.read(b)
    }
}
impl BufRead for OnConfirm {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.input.fill_buf()
    }
    fn consume(&mut self, n: usize) {
        self.input.consume(n);
    }
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        let n = self.input.read_line(buf)?;
        if buf.trim() == "y" {
            fs::write(&self.output_path, b"appeared meanwhile")?;
        }
        Ok(n)
    }
}
#[test]
fn output_created_during_confirmation_is_rechecked_before_execution() {
    let f = Fixture::new();
    let destination = f.path("output");
    let (mut p, state) = panel(
        true,
        Preview {
            outputs: vec![destination.clone()],
            ..Preview::default()
        },
    );
    let mut output = Vec::new();
    p.run(
        &[],
        &mut OnConfirm {
            input: Cursor::new("1\nr\ny\nq\n"),
            output_path: destination.clone(),
        },
        &mut output,
    )
    .unwrap();
    assert!(state.executions.borrow().is_empty());
    assert!(String::from_utf8(output).unwrap().contains("[执行未开始]"));
    assert_eq!(fs::read(destination).unwrap(), b"appeared meanwhile");
}

#[test]
fn parser_accepts_repeated_translations_and_rejects_ambiguous_values() {
    let (p, _) = panel(false, Preview::default());
    let Invocation::Explicit { parameters, .. } = p
        .parse(&args(&[
            "probe",
            "--translation",
            "别处/译文 & あ.json",
            "--translation=另一份.json",
            "--label=--文本",
        ]))
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(
        parameters.paths("translation").unwrap(),
        &[
            PathBuf::from("别处/译文 & あ.json"),
            PathBuf::from("另一份.json")
        ]
    );
    assert_eq!(parameters.text("label").unwrap(), "--文本");
    for a in [
        args(&["probe", "--label", "a", "--label", "b"]),
        args(&["probe", "--label", "--mode"]),
        args(&["probe", "--label="]),
    ] {
        assert!(p.parse(&a).is_err());
    }
}

#[test]
fn equivalent_command_quotes_powershell_literals() {
    let (p, _) = panel(false, Preview::default());
    let Invocation::Explicit { parameters, .. } = p
        .parse(&args(&[
            "probe",
            "--label",
            "汉字 &'$()",
            "--translation",
            "a b.json",
        ]))
        .unwrap()
    else {
        panic!()
    };
    let command = command_line(OsStr::new("工具 空格.exe"), &spec(false), &parameters).unwrap();
    assert_eq!(
        command,
        "& '工具 空格.exe' 'probe' '--label=汉字 &''$()' '--translation=a b.json' '--mode=普通'"
    );
}

#[test]
fn cleared_defaults_and_empty_lists_replay_without_restoring_defaults() {
    let (mut p, state) = panel(false, Preview::default());
    let log = session(&mut p, &[], "1\n3\n:clear\nc\nr\nq\n");
    assert!(log.contains("'--unset=mode'"));
    let Invocation::Explicit { parameters, .. } =
        p.parse(&args(&["probe", "--unset=mode"])).unwrap()
    else {
        panic!()
    };
    assert_eq!(state.prepared.borrow()[0], parameters);
    let mut s = spec(false);
    s.fields[1].default = Some(Value::Paths(vec![PathBuf::from("默认译文.json")]));
    let p = Panel::new(
        "test",
        "test",
        "0",
        vec![Box::new(Probe {
            state,
            spec: s.clone(),
            preview: Preview::default(),
        })],
    )
    .unwrap();
    let Invocation::Explicit { parameters, .. } =
        p.parse(&args(&["probe", "--translation="])).unwrap()
    else {
        panic!()
    };
    assert!(parameters.paths("translation").unwrap().is_empty());
    assert!(command_line(OsStr::new("test"), &s, &parameters)
        .unwrap()
        .contains("'--translation='"));
    for a in [
        args(&["probe", "--unset=translation", "--translation=x"]),
        args(&["probe", "--unset=label"]),
        args(&["probe", "--unset=unknown"]),
    ] {
        assert!(p.parse(&a).is_err());
    }
}

#[test]
fn schema_rejects_unsafe_defaults_and_missing_output_declarations() {
    let state = Rc::new(State::default());
    let mut bad = spec(true);
    bad.fields
        .push(Field::new("overwrite", "覆盖", FieldKind::Flag).default(Value::Flag(true)));
    assert!(Panel::new(
        "test",
        "test",
        "0",
        vec![Box::new(Probe {
            state,
            spec: bad,
            preview: Preview::default()
        })]
    )
    .is_err());
    let (mut p, state) = panel(true, Preview::default());
    assert!(p
        .run(&args(&["probe"]), &mut NeverRead, &mut Vec::new())
        .is_err());
    assert!(state.executions.borrow().is_empty());
}

#[test]
fn individual_menu_returns_to_home_and_eof_exits() {
    let state = Rc::new(State::default());
    let mut operation = spec(false);
    operation.primary = false;
    let mut p = Panel::new(
        "test",
        "test",
        "0",
        vec![Box::new(Probe {
            state,
            spec: operation,
            preview: Preview::default(),
        })],
    )
    .unwrap();
    let log = session(&mut p, &[], "1\n1\n0\n0\n");
    assert!(log.contains("单项操作"));
    assert!(log.matches("主菜单").count() >= 2);
}

#[test]
fn demo_binary_copies_bytes_preserves_source_and_enforces_output_policy() {
    let f = Fixture::new();
    let source = f.path("原文 & あ '源.bin");
    let output = f.path("成品 & 文.bin");
    let translations = f.path("任意译文名.json");
    fs::write(&source, [0, 1, 128, 255]).unwrap();
    fs::write(&translations, b"{}").unwrap();
    let run = |arguments: &[&OsStr]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_panel_demo"))
            .args(arguments)
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
    };
    assert!(run(&[OsStr::new("--help")]).status.success());
    assert!(run(&[
        OsStr::new("inspect"),
        OsStr::new("--source"),
        source.as_os_str(),
        OsStr::new("--translation"),
        translations.as_os_str()
    ])
    .status
    .success());
    let arguments = [
        OsStr::new("copy"),
        OsStr::new("--source"),
        source.as_os_str(),
        OsStr::new("--output"),
        output.as_os_str(),
    ];
    let copied = run(&arguments);
    assert!(
        copied.status.success(),
        "{}",
        String::from_utf8_lossy(&copied.stderr)
    );
    assert_eq!(fs::read(&output).unwrap(), [0, 1, 128, 255]);
    fs::write(&output, b"old output").unwrap();
    assert!(!run(&arguments).status.success());
    assert_eq!(fs::read(&output).unwrap(), b"old output");
    let mut overwrite = arguments.to_vec();
    overwrite.push(OsStr::new("--overwrite"));
    assert!(run(&overwrite).status.success());
    assert_eq!(fs::read(&source).unwrap(), fs::read(&output).unwrap());
    assert!(!run(&[
        OsStr::new("copy"),
        OsStr::new("--source"),
        source.as_os_str(),
        OsStr::new("--output"),
        source.as_os_str(),
        OsStr::new("--overwrite")
    ])
    .status
    .success());
    assert_eq!(fs::read(&source).unwrap(), [0, 1, 128, 255]);
    assert_eq!(fs::read(translations).unwrap(), b"{}");
    assert_eq!(fs::read_dir(&f.0).unwrap().count(), 3, "没有遗留临时文件");
    assert!(!run(&[OsStr::new("copy")]).status.success());
    assert!(!run(&[OsStr::new("unknown")]).status.success());
    assert!(Path::new(env!("CARGO_BIN_EXE_panel_demo")).exists());
}
