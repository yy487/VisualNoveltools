use crate::{
    game::{Binding, Game},
    io::{self, Inputs},
    json_bytes,
    nes::Nes,
    Result,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    RunReport, Value,
};

fn panel_error(e: impl std::fmt::Display) -> vn_cli::Error {
    vn_cli::Error(e.to_string())
}
pub enum Action {
    Inspect,
    Export,
    Import,
    Convert,
    Repair,
}
impl Operation for Action {
    fn spec(&self) -> OperationSpec {
        let (id, label) = match self {
            Self::Inspect => ("inspect", "检查引擎与脚本"),
            Self::Export => ("export", "导出原生记录和翻译 JSON"),
            Self::Import => ("import", "回注译文并扩容 PRG"),
            Self::Convert => ("convert", "原始 FDS 转为 MMC5 NES"),
            Self::Repair => ("repair", "只修补 NES 驱动代码"),
        };
        let mut fields = vec![Field::new(
            if matches!(self, Self::Import) {
                "workspace"
            } else {
                "source"
            },
            if matches!(self, Self::Import) {
                "导出工作区"
            } else {
                "FDS 或 MMC5 NES"
            },
            FieldKind::Path,
        )
        .required()];
        if matches!(self, Self::Convert) {
            fields.push(Field::new("bios", "8KB disksys.rom", FieldKind::Path).required());
        }
        if matches!(self, Self::Import) {
            fields.push(Field::new(
                "translation",
                "独立译后 JSON（可多选，须携带 binding.json）",
                FieldKind::Paths,
            ));
            fields.push(Field::new(
                "font",
                "Windows 字体名称，默认黑体 SimHei",
                FieldKind::Text,
            ));
        }
        if !matches!(self, Self::Inspect) {
            fields.push(
                Field::new("output", "新的输出目录（父目录须存在）", FieldKind::Path).required(),
            );
        }
        let mut s = OperationSpec::new(id, label, fields);
        s.writes = !matches!(self, Self::Inspect);
        s.primary = !matches!(self, Self::Inspect);
        s.description="支持已验证水晶之龙 FDS 原生文本，以及首句 MMC5 NES / 本工具 V3 的 16×16 中文、分页和 PRG 扩容；输出拒绝覆盖。".into();
        s
    }
    fn prefill(&self, paths: &[PathBuf], p: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(path) = paths.first() {
            p.set(
                if matches!(self, Self::Import) {
                    "workspace"
                } else {
                    "source"
                },
                Value::Path(path.clone()),
            );
        }
        Ok(())
    }
    fn prepare(&self, p: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        self.prepare_job(p)
            .map(|job| Box::new(job) as Box<dyn PreparedOperation>)
            .map_err(panel_error)
    }
}
impl Action {
    fn prepare_job(&self, p: &Parameters) -> Result<Job> {
        if p.flag("overwrite") {
            return Err("本工具不支持覆盖；请选择新的输出目录".into());
        }
        let mut inputs = Inputs::default();
        let mut files = BTreeMap::new();
        let mut output = None;
        let summary;
        if matches!(self, Self::Repair) {
            let bytes = inputs.read(p.path("source")?)?;
            let old = Nes::parse(&bytes)?;
            let result = old.repair_runtime()?;
            let new = Nes::parse(&result.image)?;
            files = new.export_files()?;
            summary = "驱动修补完成；原正文和字模未重编码、未迁移".into();
            files.insert("verification.json".into(),json_bytes(&serde_json::json!({"operation":"repair_runtime","source_sha256":crate::sha256(&bytes),"output_sha256":crate::sha256(&result.image),"messages_unchanged":true,"glyphs_unchanged":true,"translation_migration":false}))?);
        } else if matches!(self, Self::Convert) {
            let source = inputs.read(p.path("source")?)?;
            let bios = inputs.read(p.path("bios")?)?;
            let game = Nes::convert(&source, &bios)?;
            summary = format!("FDS 资源转换完成；{}", game.summary());
            files = game.export_files()?;
            files.insert("archive/original.fds".into(), source);
            files.insert("verification.json".into(),json_bytes(&serde_json::json!({"source_format":"FDS","output_format":"NES 2.0 MMC5","native_texts_unchanged":true,"scenes_unchanged":true,"audio_save_fullgame":"separate compatibility work"}))?);
        } else if matches!(self, Self::Import) {
            let root = p.path("workspace")?;
            inputs.protect(root)?;
            let nes = root.join("source.nes");
            let fds = root.join("source.fds");
            let bytes = inputs.read(if nes.try_exists()? { &nes } else { &fds })?;
            let native = inputs.read(&root.join("native.json"))?;
            let binding: Binding =
                serde_json::from_slice(&inputs.read(&root.join("translations/binding.json"))?)?;
            let mut selected = match p.get("translation") {
                Some(Value::Paths(paths)) => paths.clone(),
                _ => Vec::new(),
            };
            let supplied = if bytes.starts_with(b"NES\x1A") {
                let game = Nes::parse(&bytes)?;
                verify_binding(
                    &binding,
                    &game.binding()?,
                    &native,
                    &json_bytes(&game.native)?,
                )?;
                if selected.is_empty() {
                    selected = game
                        .native
                        .resources
                        .iter()
                        .map(|r| root.join(format!("translations/{}.json", r.name)))
                        .collect();
                }
                let translations = load_translations(&mut inputs, &selected, &binding)?;
                let face = match p.get("font") {
                    Some(Value::Text(s)) if !s.is_empty() => s.as_str(),
                    _ => "SimHei",
                };
                let rebuilt = game.import(&translations, face)?;
                let reparsed = Nes::parse(&rebuilt.image)?;
                files = reparsed.export_files()?;
                summary = format!(
                    "{}；回注 {} 条文本",
                    reparsed.summary(),
                    rebuilt.changed_entries
                );
                rebuilt
            } else {
                let game = Game::parse(&bytes)?;
                verify_binding(
                    &binding,
                    &game.binding()?,
                    &native,
                    &json_bytes(&game.native)?,
                )?;
                if selected.is_empty() {
                    selected = game
                        .native
                        .resources
                        .iter()
                        .map(|r| root.join(format!("translations/{}.json", r.name)))
                        .collect();
                }
                let translations = load_translations(&mut inputs, &selected, &binding)?;
                let rebuilt = game.import(&translations)?;
                files = Game::parse(&rebuilt.image)?.export_files()?;
                summary = format!("FDS 原生编码；回注 {} 条文本", rebuilt.changed_entries);
                rebuilt
            };
            files.insert("verification.json".into(),json_bytes(&serde_json::json!({"changed_entries":supplied.changed_entries,"grown_data":supplied.grown_files,"output_sha256":crate::sha256(&supplied.image),"reextract_matches":true,"scenes_unchanged":true,"compatibility_unverified":["FDS wavetable audio","save/load UI and persistence","full-game playthrough"]}))?);
        } else {
            let bytes = inputs.read(p.path("source")?)?;
            if bytes.starts_with(b"NES\x1A") {
                let game = Nes::parse(&bytes)?;
                summary = game.summary();
                if matches!(self, Self::Export) {
                    files = game.export_files()?;
                }
            } else {
                let game = Game::parse(&bytes)?;
                summary = game.summary();
                if matches!(self, Self::Export) {
                    files = game.export_files()?;
                }
            }
        }
        if !matches!(self, Self::Inspect) {
            output = Some(inputs.output(p.path("output")?)?);
        }
        let preview = Preview {
            inputs: inputs.roots.clone(),
            outputs: output.clone().into_iter().collect(),
            details: vec![summary.clone()],
            steps: vec![
                "重新解析来源、校验模板和引擎".into(),
                "重提取核对后暂存、提交独立目录".into(),
            ],
        };
        Ok(Job {
            inputs,
            files,
            output,
            preview,
            summary,
        })
    }
}
fn verify_binding(got: &Binding, expected: &Binding, native: &[u8], parsed: &[u8]) -> Result<()> {
    if got != expected || native != parsed {
        return Err("来源绑定或完整原生记录不符；请从当前源重新导出".into());
    }
    Ok(())
}
fn load_translations(
    inputs: &mut Inputs,
    paths: &[PathBuf],
    binding: &Binding,
) -> Result<BTreeMap<String, Vec<vn_text::Entry>>> {
    let mut result = BTreeMap::new();
    for path in paths {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("译文文件名非法")?;
        if !binding.templates.contains_key(name) {
            return Err(format!("未绑定翻译文件: {name}").into());
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let actual: Binding = serde_json::from_slice(&inputs.read(&parent.join("binding.json"))?)?;
        if &actual != binding {
            return Err(format!("{name} 所在目录的 binding.json 来源不符").into());
        }
        let entries = vn_text::read_template(&inputs.read(path)?)?;
        if result
            .insert(name.trim_end_matches(".json").to_owned(), entries)
            .is_some()
        {
            return Err("重复选择同一翻译单元".into());
        }
    }
    Ok(result)
}
struct Job {
    inputs: Inputs,
    files: BTreeMap<String, Vec<u8>>,
    output: Option<PathBuf>,
    preview: Preview,
    summary: String,
}
impl PreparedOperation for Job {
    fn preview(&self) -> &Preview {
        &self.preview
    }
    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        self.inputs.verify().map_err(panel_error)?;
        if let Some(output) = &self.output {
            progress.report("提交已重提取验证的输出")?;
            io::publish(&self.inputs, output, &self.files).map_err(panel_error)?;
        }
        Ok(RunReport {
            summary: self.summary,
            outputs: self.output.into_iter().collect(),
            totals: vec![("输出文件".into(), self.files.len() as u64)],
            ..RunReport::default()
        })
    }
}
