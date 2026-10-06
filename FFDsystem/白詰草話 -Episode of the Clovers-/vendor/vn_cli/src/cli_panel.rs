//! A line-oriented, stream-testable panel. Format parsing and output transactions
//! belong to each operation, not to the menu layer.
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{self, BufRead, Write};
use std::path::{Component, Path, PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(pub String);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self(value.to_string())
    }
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Path(PathBuf),
    Paths(Vec<PathBuf>),
    Text(String),
    Flag(bool),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parameters(BTreeMap<String, Value>);
impl Parameters {
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.0.insert(key.into(), value);
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }
    pub fn path(&self, key: &str) -> Result<&Path> {
        match self.get(key) {
            Some(Value::Path(p)) => Ok(p),
            _ => Err(format!("缺少路径参数 --{key}").into()),
        }
    }
    pub fn paths(&self, key: &str) -> Result<&[PathBuf]> {
        match self.get(key) {
            Some(Value::Paths(p)) => Ok(p),
            _ => Err(format!("缺少路径列表 --{key}").into()),
        }
    }
    pub fn text(&self, key: &str) -> Result<&str> {
        match self.get(key) {
            Some(Value::Text(v)) => Ok(v),
            _ => Err(format!("缺少文本参数 --{key}").into()),
        }
    }
    pub fn flag(&self, key: &str) -> bool {
        matches!(self.get(key), Some(Value::Flag(true)))
    }
}

#[derive(Clone, Debug)]
pub enum FieldKind {
    Path,
    Paths,
    Text,
    Flag,
    Choice(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub id: String,
    pub label: String,
    pub kind: FieldKind,
    pub required: bool,
    pub default: Option<Value>,
}
impl Field {
    pub fn new(id: &str, label: &str, kind: FieldKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            required: false,
            default: None,
        }
    }
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    pub fn default(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }
}

#[derive(Clone, Debug)]
pub struct OperationSpec {
    pub id: String,
    pub title: String,
    pub description: String,
    /// Primary operations appear on the home menu; others under 单项操作.
    pub primary: bool,
    pub writes: bool,
    pub fields: Vec<Field>,
}
impl OperationSpec {
    pub fn new(id: &str, title: &str, fields: Vec<Field>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            primary: true,
            writes: true,
            fields,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Preview {
    /// All source files/trees, selected translations and other protected inputs.
    pub inputs: Vec<PathBuf>,
    /// Final files/directories written by the operation (not individual descendants).
    pub outputs: Vec<PathBuf>,
    pub steps: Vec<String>,
    pub details: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct RunReport {
    pub summary: String,
    pub totals: Vec<(String, u64)>,
    pub outputs: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

pub trait Progress {
    fn report(&mut self, message: &str) -> Result<()>;
}
struct TextProgress<'a>(&'a mut dyn Write);
impl Progress for TextProgress<'_> {
    fn report(&mut self, message: &str) -> Result<()> {
        writeln!(self.0, "[进度] {message}")?;
        self.0.flush()?;
        Ok(())
    }
}

/// A prepared job owns its validated data. Execution must use that snapshot or
/// revalidate external inputs, commit atomically and protect concurrent output changes.
pub trait PreparedOperation {
    fn preview(&self) -> &Preview;
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport>;
}

pub trait Operation {
    fn spec(&self) -> OperationSpec;
    /// Suggestions only. Do not create files or overwrite explicit user values.
    fn prefill(&self, _paths: &[PathBuf], _parameters: &mut Parameters) -> Result<()> {
        Ok(())
    }
    /// Read-only validation/preparation, shared by CLI execution and the panel.
    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>>;
}

struct Registered {
    operation: Box<dyn Operation>,
    spec: OperationSpec,
    parameters: Parameters,
    prefilled: bool,
}

/// One instance represents one session. No process-global state or terminal modes.
pub struct Panel {
    executable: OsString,
    title: String,
    version: String,
    operations: Vec<Registered>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invocation {
    Help(Option<usize>),
    Interactive(Vec<PathBuf>),
    Explicit {
        operation: usize,
        parameters: Parameters,
    },
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Flow {
    Back,
    Exit,
}

impl Panel {
    pub fn new(
        executable: impl Into<OsString>,
        title: &str,
        version: &str,
        operations: Vec<Box<dyn Operation>>,
    ) -> Result<Self> {
        if operations.is_empty() {
            return Err("至少需要注册一个已实现的操作".into());
        }
        let mut ids = BTreeSet::new();
        let mut registered = Vec::new();
        for operation in operations {
            let mut spec = operation.spec();
            if !valid_id(&spec.id) || !ids.insert(spec.id.clone()) {
                return Err(format!("无效或重复的操作 ID: {}", spec.id).into());
            }
            if spec.writes && !spec.fields.iter().any(|f| f.id == "overwrite") {
                spec.fields.push(
                    Field::new("overwrite", "覆盖已有输出", FieldKind::Flag)
                        .default(Value::Flag(false)),
                );
            }
            let mut field_ids = BTreeSet::new();
            for field in &spec.fields {
                if !valid_id(&field.id)
                    || matches!(field.id.as_str(), "help" | "version" | "unset")
                    || !field_ids.insert(field.id.clone())
                {
                    return Err(format!("无效或重复的参数 ID: {}", field.id).into());
                }
                if field.id == "overwrite" && !matches!(field.kind, FieldKind::Flag) {
                    return Err("overwrite 必须是布尔参数".into());
                }
                if field.id == "overwrite" && matches!(field.default, Some(Value::Flag(true))) {
                    return Err("overwrite 默认值必须为 false".into());
                }
                if let FieldKind::Choice(values) = &field.kind {
                    let unique: BTreeSet<_> = values.iter().collect();
                    if values.is_empty()
                        || unique.len() != values.len()
                        || values.iter().any(|v| v.trim().is_empty())
                    {
                        return Err(format!("无效的选项列表: {}", field.id).into());
                    }
                }
                if let Some(value) = &field.default {
                    validate_value(field, value)?;
                }
            }
            let parameters = defaults(&spec);
            registered.push(Registered {
                operation,
                spec,
                parameters,
                prefilled: false,
            });
        }
        Ok(Self {
            executable: executable.into(),
            title: title.into(),
            version: version.into(),
            operations: registered,
        })
    }

    /// Args exclude argv[0]. `-- PATH...` explicitly requests path-prefilled mode.
    pub fn parse(&self, args: &[OsString]) -> Result<Invocation> {
        if args.is_empty() {
            return Ok(Invocation::Interactive(Vec::new()));
        }
        if args[0] == "--help" || args[0] == "-h" {
            return Ok(Invocation::Help(None));
        }
        if args[0] == "--" {
            return Ok(Invocation::Interactive(
                args[1..].iter().map(PathBuf::from).collect(),
            ));
        }
        if let Some(index) = self
            .operations
            .iter()
            .position(|op| args[0] == op.spec.id.as_str())
        {
            if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
                return Ok(Invocation::Help(Some(index)));
            }
            return Ok(Invocation::Explicit {
                operation: index,
                parameters: parse_fields(&self.operations[index].spec, &args[1..])?,
            });
        }
        let path_only = args.iter().all(|a| !a.to_string_lossy().starts_with('-'));
        let first = Path::new(&args[0]);
        let looks_like_path = first.exists() || args[0].to_string_lossy().contains(['/', '\\']);
        if path_only && looks_like_path {
            return Ok(Invocation::Interactive(
                args.iter().map(PathBuf::from).collect(),
            ));
        }
        Err(format!(
            "未知命令 {}；使用 --help，或用 -- <路径> 进入面板",
            args[0].to_string_lossy()
        )
        .into())
    }

    pub fn run_env(&mut self) -> Result<()> {
        init_console_utf8();
        let args = std::env::args_os().skip(1).collect::<Vec<_>>();
        self.run(&args, &mut io::stdin().lock(), &mut io::stdout().lock())
    }

    /// Fully specified commands never read from `input`. Interactive errors stay
    /// in the session; explicit failures are returned to the caller for nonzero exit.
    pub fn run(
        &mut self,
        args: &[OsString],
        input: &mut dyn BufRead,
        output: &mut dyn Write,
    ) -> Result<()> {
        match self.parse(args)? {
            Invocation::Help(index) => self.help(index, output),
            Invocation::Explicit {
                operation,
                parameters,
            } => {
                let job = self.prepare(operation, &parameters)?;
                check_outputs(&self.operations[operation].spec, &parameters, job.preview())?;
                let report = job.execute(&mut TextProgress(output))?;
                print_report(output, &report)
            }
            Invocation::Interactive(paths) => self.interactive(&paths, input, output),
        }
    }

    fn help(&self, index: Option<usize>, output: &mut dyn Write) -> Result<()> {
        writeln!(output, "{} {}", self.title, self.version)?;
        writeln!(
            output,
            "用法: {} <操作> [参数]\n      {} [--] [路径 ...]",
            self.executable.to_string_lossy(),
            self.executable.to_string_lossy()
        )?;
        for (i, op) in self.operations.iter().enumerate() {
            if index.is_some_and(|n| n != i) {
                continue;
            }
            writeln!(
                output,
                "\n{} — {}\n  {}",
                op.spec.id, op.spec.title, op.spec.description
            )?;
            for field in &op.spec.fields {
                let kind = match field.kind {
                    FieldKind::Flag => "开关",
                    FieldKind::Paths => "路径（可重复）",
                    FieldKind::Path => "路径",
                    FieldKind::Text => "文本",
                    FieldKind::Choice(_) => "选项",
                };
                write!(
                    output,
                    "  --{} <{}>  {}{}",
                    field.id,
                    kind,
                    field.label,
                    if field.required { " [必需]" } else { "" }
                )?;
                if let FieldKind::Choice(values) = &field.kind {
                    write!(output, " [{}]", values.join(" | "))?;
                }
                if let Some(value) = &field.default {
                    write!(output, " [默认: {}]", display_value(value))?;
                }
                writeln!(output)?;
            }
        }
        writeln!(output, "\n完整命令执行一次，不询问输入；无参数/仅路径进入持续面板。\n布尔参数支持 --flag 或 --flag=false；以 -- 开头的值使用 --key=value。")?;
        writeln!(
            output,
            "清除默认参数: --unset=<参数名>；清空路径列表: --key=。"
        )?;
        Ok(())
    }

    fn prepare(&self, index: usize, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let op = &self.operations[index];
        validate_parameters(&op.spec, parameters)?;
        let job = op.operation.prepare(parameters)?;
        check_outputs(&op.spec, parameters, job.preview())?;
        Ok(job)
    }

    fn interactive(
        &mut self,
        paths: &[PathBuf],
        input: &mut dyn BufRead,
        output: &mut dyn Write,
    ) -> Result<()> {
        let mut single = false;
        loop {
            writeln!(
                output,
                "\n{} / {}\n{}",
                self.title,
                if single { "单项操作" } else { "主菜单" },
                "─".repeat(36)
            )?;
            let indices = self
                .operations
                .iter()
                .enumerate()
                .filter(|(_, op)| {
                    if single {
                        !op.spec.primary
                    } else {
                        op.spec.primary
                    }
                })
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            for (n, i) in indices.iter().enumerate() {
                let spec = &self.operations[*i].spec;
                writeln!(output, "[{}] {}", n + 1, spec.title)?;
                if !spec.description.is_empty() {
                    writeln!(output, "    {}", spec.description)?;
                }
            }
            let has_single = !single && self.operations.iter().any(|op| !op.spec.primary);
            if has_single {
                writeln!(output, "[{}] 单项操作", indices.len() + 1)?;
            }
            writeln!(
                output,
                "[0] {}  [Q] 退出",
                if single { "返回" } else { "退出" }
            )?;
            let Some(line) = prompt(input, output, "选择: ")? else {
                return Ok(());
            };
            let choice = line.trim();
            if choice.eq_ignore_ascii_case("q") {
                return Ok(());
            }
            if choice == "0" {
                if single {
                    single = false;
                    continue;
                }
                return Ok(());
            }
            let Some(n) = choice.parse::<usize>().ok().filter(|n| *n > 0) else {
                writeln!(output, "请输入菜单编号。")?;
                continue;
            };
            if has_single && n == indices.len() + 1 {
                single = true;
                continue;
            }
            let Some(index) = indices.get(n - 1).copied() else {
                writeln!(output, "没有这个操作。")?;
                continue;
            };
            if !self.operations[index].prefilled {
                let op = &mut self.operations[index];
                // Commit suggestions only when the entire prefill hook succeeds.
                let mut suggested = op.parameters.clone();
                match op.operation.prefill(paths, &mut suggested) {
                    Ok(()) => op.parameters = suggested,
                    Err(error) => writeln!(output, "[预填失败] {error}；可手动编辑参数。")?,
                }
                op.prefilled = true;
            }
            if self.operation_menu(index, input, output)? == Flow::Exit {
                return Ok(());
            }
        }
    }

    fn operation_menu(
        &mut self,
        index: usize,
        input: &mut dyn BufRead,
        output: &mut dyn Write,
    ) -> Result<Flow> {
        loop {
            let op = &self.operations[index];
            writeln!(
                output,
                "\n{} / {}\n{}",
                self.title,
                op.spec.title,
                "─".repeat(36)
            )?;
            for (n, field) in op.spec.fields.iter().enumerate() {
                writeln!(
                    output,
                    "[{}] {}: {}",
                    n + 1,
                    field.label,
                    op.parameters
                        .get(&field.id)
                        .map(display_value)
                        .unwrap_or_else(|| "未设置".into())
                )?;
            }
            writeln!(
                output,
                "[P] 预检  [R] 执行  [C] 等价命令  [0] 返回  [Q] 退出"
            )?;
            let Some(line) = prompt(input, output, "选择: ")? else {
                return Ok(Flow::Exit);
            };
            match line.trim().to_ascii_lowercase().as_str() {
                "0" => return Ok(Flow::Back),
                "q" => return Ok(Flow::Exit),
                "c" => match command_line(&self.executable, &op.spec, &op.parameters) {
                    Ok(cmd) => writeln!(output, "PowerShell 7.3+（Standard 参数模式）: {cmd}")?,
                    Err(e) => writeln!(output, "[命令不可用] {e}")?,
                },
                "p" | "r" => {
                    // A previewed job is never cached: R prepares a fresh job.
                    let job = match self.prepare(index, &op.parameters) {
                        Ok(job) => job,
                        Err(error) => {
                            writeln!(output, "[预检失败] {error}")?;
                            continue;
                        }
                    };
                    print_preview(output, job.preview())?;
                    if line.trim().eq_ignore_ascii_case("p") {
                        continue;
                    }
                    if op.spec.writes {
                        writeln!(
                            output,
                            "覆盖已有输出: {}",
                            if op.parameters.flag("overwrite") {
                                "允许"
                            } else {
                                "不允许"
                            }
                        )?;
                        let Some(answer) = prompt(
                            input,
                            output,
                            "[Y] 确认执行 / [0] 返回修改 / [Q] 退出（默认取消）: ",
                        )?
                        else {
                            return Ok(Flow::Exit);
                        };
                        if answer.trim().eq_ignore_ascii_case("q") {
                            return Ok(Flow::Exit);
                        }
                        if !matches!(
                            answer.trim().to_ascii_lowercase().as_str(),
                            "y" | "yes" | "是"
                        ) {
                            writeln!(output, "已取消，参数保留。").map_err(Error::from)?;
                            continue;
                        }
                    }
                    if let Err(error) = check_outputs(&op.spec, &op.parameters, job.preview()) {
                        writeln!(output, "[执行未开始] {error}")?;
                        continue;
                    }
                    match job.execute(&mut TextProgress(output)) {
                        Ok(report) => {
                            print_report(output, &report)?;
                            return Ok(Flow::Back);
                        }
                        Err(error) => {
                            writeln!(output, "[操作失败] {error}\n参数保留，可修改后重试。")?
                        }
                    }
                }
                value => {
                    let Some(n) = value
                        .parse::<usize>()
                        .ok()
                        .filter(|n| *n > 0 && *n <= op.spec.fields.len())
                    else {
                        writeln!(output, "请输入参数编号或 P/R/C/0/Q。")?;
                        continue;
                    };
                    let field = op.spec.fields[n - 1].clone();
                    if edit_field(
                        input,
                        output,
                        &field,
                        &mut self.operations[index].parameters,
                    )? == Flow::Exit
                    {
                        return Ok(Flow::Exit);
                    }
                }
            }
        }
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.as_bytes()[0].is_ascii_lowercase()
        && id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn defaults(spec: &OperationSpec) -> Parameters {
    let mut values = Parameters::default();
    for field in &spec.fields {
        if let Some(value) = &field.default {
            values.set(&field.id, value.clone());
        } else if matches!(field.kind, FieldKind::Flag) {
            values.set(&field.id, Value::Flag(false));
        } else if matches!(field.kind, FieldKind::Paths) {
            values.set(&field.id, Value::Paths(Vec::new()));
        }
    }
    values
}
fn validate_value(field: &Field, value: &Value) -> Result<()> {
    let valid = match (&field.kind, value) {
        (FieldKind::Path, Value::Path(_))
        | (FieldKind::Paths, Value::Paths(_))
        | (FieldKind::Text, Value::Text(_))
        | (FieldKind::Flag, Value::Flag(_)) => true,
        (FieldKind::Choice(choices), Value::Text(text)) => choices.contains(text),
        _ => false,
    };
    if !valid {
        return Err(format!("--{} 的值类型或选项无效", field.id).into());
    }
    let empty = match value {
        Value::Path(p) => p.as_os_str().is_empty(),
        Value::Paths(p) => p.is_empty() || p.iter().any(|p| p.as_os_str().is_empty()),
        Value::Text(s) => s.trim().is_empty(),
        Value::Flag(_) => false,
    };
    if field.required && empty {
        return Err(format!("--{} 不能为空", field.id).into());
    }
    Ok(())
}
fn validate_parameters(spec: &OperationSpec, parameters: &Parameters) -> Result<()> {
    for key in parameters.0.keys() {
        if !spec.fields.iter().any(|f| &f.id == key) {
            return Err(format!("未知参数 --{key}").into());
        }
    }
    for field in &spec.fields {
        match parameters.get(&field.id) {
            Some(value) => validate_value(field, value)?,
            None if field.required => return Err(format!("缺少必需参数 --{}", field.id).into()),
            None => {}
        }
    }
    Ok(())
}
fn parse_value(field: &Field, raw: OsString) -> Result<Value> {
    Ok(match &field.kind {
        FieldKind::Path => Value::Path(PathBuf::from(raw)),
        FieldKind::Paths => Value::Paths(if raw.is_empty() {
            Vec::new()
        } else {
            vec![PathBuf::from(raw)]
        }),
        FieldKind::Flag => Value::Flag(match raw.to_str() {
            Some("true" | "1") => true,
            Some("false" | "0") => false,
            _ => return Err(format!("--{} 需要 true 或 false", field.id).into()),
        }),
        FieldKind::Text | FieldKind::Choice(_) => Value::Text(
            raw.into_string()
                .map_err(|_| Error(format!("--{} 不是有效 Unicode 文本", field.id)))?,
        ),
    })
}
fn parse_fields(spec: &OperationSpec, args: &[OsString]) -> Result<Parameters> {
    let mut result = defaults(spec);
    let mut seen = BTreeSet::new();
    let mut unset = BTreeSet::new();
    let mut i = 0;
    while i < args.len() {
        let token = args[i].to_str().ok_or("参数名不是有效 Unicode")?;
        let option = token
            .strip_prefix("--")
            .ok_or_else(|| Error(format!("预期命名参数，实际为 {token}")))?;
        let (id, inline) = option
            .split_once('=')
            .map_or((option, None), |(a, b)| (a, Some(b)));
        if id == "unset" {
            let target = if let Some(target) = inline {
                target
            } else {
                i += 1;
                args.get(i)
                    .and_then(|v| v.to_str())
                    .ok_or("--unset 缺少参数名")?
            };
            if !spec.fields.iter().any(|f| f.id == target) {
                return Err(format!("未知参数 --{target}").into());
            }
            if !seen.insert(target.to_owned()) {
                return Err(format!("参数重复 --{target}").into());
            }
            unset.insert(target.to_owned());
            result.0.remove(target);
            i += 1;
            continue;
        }
        let field = spec
            .fields
            .iter()
            .find(|f| f.id == id)
            .ok_or_else(|| Error(format!("未知参数 --{id}")))?;
        let first = seen.insert(id.to_string());
        if unset.contains(id) || (!first && !matches!(field.kind, FieldKind::Paths)) {
            return Err(format!("参数重复 --{id}").into());
        }
        let raw = if let Some(value) = inline {
            OsString::from(value)
        } else if matches!(field.kind, FieldKind::Flag) {
            OsString::from("true")
        } else {
            i += 1;
            let value = args.get(i).ok_or_else(|| Error(format!("--{id} 缺少值")))?;
            if value.to_string_lossy().starts_with("--") {
                return Err(format!("--{id} 缺少值；以 -- 开头的值请用 --{id}=<值>").into());
            }
            value.clone()
        };
        let value = parse_value(field, raw)?;
        if let Value::Paths(mut incoming) = value {
            if first {
                result.set(id, Value::Paths(Vec::new()));
            }
            if let Some(Value::Paths(existing)) = result.0.get_mut(id) {
                existing.append(&mut incoming);
            }
        } else {
            result.set(id, value);
        }
        i += 1;
    }
    validate_parameters(spec, &result)?;
    Ok(result)
}

fn display_value(value: &Value) -> String {
    match value {
        Value::Path(p) => p.display().to_string(),
        Value::Paths(paths) => paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n    "),
        Value::Text(t) => t.clone(),
        Value::Flag(b) => if *b { "是" } else { "否" }.into(),
    }
}
fn prompt(input: &mut dyn BufRead, output: &mut dyn Write, label: &str) -> Result<Option<String>> {
    write!(output, "{label}")?;
    output.flush()?;
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim_end_matches(['\r', '\n']).to_owned()))
}
fn unquote_path(value: &str) -> &str {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}
fn edit_field(
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    field: &Field,
    parameters: &mut Parameters,
) -> Result<Flow> {
    writeln!(
        output,
        "\n修改 {}（空行保留，:cancel 取消，:clear 清除，:quit 退出）",
        field.label
    )?;
    if let FieldKind::Choice(choices) = &field.kind {
        writeln!(output, "可选: {}", choices.join(" / "))?;
    }
    if matches!(field.kind, FieldKind::Paths) {
        writeln!(output, "每行一个路径；最后输入空行保存。")?;
    }
    let mut paths = Vec::new();
    loop {
        let Some(line) = prompt(input, output, "> ")? else {
            return Ok(Flow::Exit);
        };
        match line.trim() {
            ":quit" => return Ok(Flow::Exit),
            ":cancel" => return Ok(Flow::Back),
            ":clear" => {
                parameters.0.remove(&field.id);
                return Ok(Flow::Back);
            }
            "" => {
                if !paths.is_empty() {
                    parameters.set(&field.id, Value::Paths(paths));
                }
                return Ok(Flow::Back);
            }
            _ => {}
        }
        if matches!(field.kind, FieldKind::Paths) {
            paths.push(PathBuf::from(unquote_path(&line)));
            continue;
        }
        let value = match field.kind {
            FieldKind::Flag => match line.trim().to_ascii_lowercase().as_str() {
                "y" | "yes" | "true" | "1" | "是" => Value::Flag(true),
                "n" | "no" | "false" | "0" | "否" => Value::Flag(false),
                _ => {
                    writeln!(output, "请输入 y 或 n。")?;
                    continue;
                }
            },
            FieldKind::Path => Value::Path(PathBuf::from(unquote_path(&line))),
            _ => Value::Text(line),
        };
        if let Err(error) = validate_value(field, &value) {
            writeln!(output, "[输入无效] {error}")?;
            continue;
        }
        parameters.set(&field.id, value);
        return Ok(Flow::Back);
    }
}
fn print_preview(output: &mut dyn Write, preview: &Preview) -> Result<()> {
    writeln!(output, "\n[预检通过]")?;
    for path in &preview.inputs {
        writeln!(output, "输入: {}", path.display())?;
    }
    for (i, step) in preview.steps.iter().enumerate() {
        writeln!(output, "{}. {step}", i + 1)?;
    }
    for detail in &preview.details {
        writeln!(output, "{detail}")?;
    }
    for path in &preview.outputs {
        writeln!(output, "输出: {}", path.display())?;
    }
    Ok(())
}
fn print_report(output: &mut dyn Write, report: &RunReport) -> Result<()> {
    writeln!(output, "\n[完成] {}", report.summary)?;
    for (label, value) in &report.totals {
        writeln!(output, "{label}: {value}")?;
    }
    for warning in &report.warnings {
        writeln!(output, "[警告] {warning}")?;
    }
    for path in &report.outputs {
        writeln!(output, "输出: {}", path.display())?;
    }
    Ok(())
}

/// Quote argv for PowerShell 7.3+ with Standard native argument passing.
/// Windows PowerShell 5.1's legacy argv rewriting is not supported.
/// No shell is launched by the panel.
pub fn command_line(
    executable: &OsStr,
    spec: &OperationSpec,
    parameters: &Parameters,
) -> Result<String> {
    validate_parameters(spec, parameters)?;
    let quote = |s: &OsStr| -> Result<String> {
        Ok(format!(
            "'{}'",
            s.to_str()
                .ok_or("路径不是有效 Unicode，无法生成可复制命令")?
                .replace('\'', "''")
        ))
    };
    let mut args = vec![
        format!("& {}", quote(executable)?),
        quote(OsStr::new(&spec.id))?,
    ];
    for field in &spec.fields {
        let Some(value) = parameters.get(&field.id) else {
            args.push(quote(OsStr::new(&format!("--unset={}", field.id)))?);
            continue;
        };
        let values = match value {
            Value::Path(p) => vec![p.as_os_str().to_owned()],
            Value::Paths(paths) if paths.is_empty() => vec![OsString::new()],
            Value::Paths(paths) => paths.iter().map(|p| p.as_os_str().to_owned()).collect(),
            Value::Text(t) => vec![OsString::from(t)],
            Value::Flag(v) => vec![OsString::from(if *v { "true" } else { "false" })],
        };
        for value in values {
            let mut arg = OsString::from(format!("--{}=", field.id));
            arg.push(value);
            args.push(quote(&arg)?);
        }
    }
    Ok(args.join(" "))
}

fn resolved(path: &Path) -> Result<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err("输出或输入路径为空".into());
    }
    let mut ancestor = std::path::absolute(path)?;
    let mut missing = Vec::new();
    // Start at the requested path: permission to traverse an ancestor need not
    // include permission to open it. Resolve the nearest existing ancestor once.
    let mut result = loop {
        match std::fs::canonicalize(&ancestor) {
            Ok(canonical) => break canonical,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let component = ancestor.components().next_back().ok_or("无法解析路径根")?;
                if !matches!(
                    component,
                    Component::Normal(_) | Component::ParentDir | Component::CurDir
                ) {
                    return Err(format!("路径根不存在: {}", ancestor.display()).into());
                }
                missing.push(component.as_os_str().to_owned());
                ancestor.pop();
            }
            Err(error) => {
                return Err(format!("无法解析路径 {}: {error}", ancestor.display()).into())
            }
        }
    };
    for part in missing.into_iter().rev() {
        if part == ".." {
            result.pop();
        } else if part != "." {
            result.push(part);
        }
    }
    Ok(result)
}
fn overlaps(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    let (a, b) = (
        PathBuf::from(a.as_os_str().to_string_lossy().to_lowercase()),
        PathBuf::from(b.as_os_str().to_string_lossy().to_lowercase()),
    );
    a.starts_with(&b) || b.starts_with(&a)
}
fn check_outputs(spec: &OperationSpec, parameters: &Parameters, preview: &Preview) -> Result<()> {
    if spec.writes && preview.outputs.is_empty() {
        return Err("写入操作必须声明最终输出路径".into());
    }
    if !spec.writes && !preview.outputs.is_empty() {
        return Err("只读操作不能声明写入输出；请修正操作注册".into());
    }
    let inputs = preview
        .inputs
        .iter()
        .map(|p| resolved(p))
        .collect::<Result<Vec<_>>>()?;
    let mut outputs: Vec<PathBuf> = Vec::new();
    for output in &preview.outputs {
        let normalized = resolved(output)?;
        if inputs.iter().any(|input| overlaps(input, &normalized)) {
            return Err(format!("输出与受保护输入重叠: {}", output.display()).into());
        }
        if outputs.iter().any(|prior| overlaps(prior, &normalized)) {
            return Err(format!("重复或重叠的输出: {}", output.display()).into());
        }
        if output.try_exists()? && !parameters.flag("overwrite") {
            return Err(format!(
                "输出已存在，默认不覆盖: {}；可修改路径或允许 --overwrite",
                output.display()
            )
            .into());
        }
        outputs.push(normalized);
    }
    Ok(())
}

#[cfg(windows)]
fn init_console_utf8() {
    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleCP(page: u32) -> i32;
        fn SetConsoleOutputCP(page: u32) -> i32;
    }
    // These calls only select the console's encoding, never alter its input mode.
    unsafe {
        SetConsoleCP(65001);
        SetConsoleOutputCP(65001);
    }
}
#[cfg(not(windows))]
fn init_console_utf8() {}
