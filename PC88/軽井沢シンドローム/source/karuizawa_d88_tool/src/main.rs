use karuizawa_d88_tool::workflow::{
    prepare_localized_build, prepare_translation_export, PreparedLocalizedBuild,
    PreparedTranslationExport,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation, Preview,
    Progress, Result, RunReport, Value,
};

struct ExportTranslation;

impl Operation for ExportTranslation {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export",
            "导出正式翻译 JSON",
            vec![
                Field::new("workspace", "已有 N88 实文件解包目录", FieldKind::Path).required(),
                Field::new("output", "独立翻译 JSON 输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "从已有 work/unpack 实文件读取八个 N88-BASIC 脚本；过滤空串和纯控制串，把半角片假名转为全角，并导出控制符已剥离的可编辑 message。不会重复解包 D88。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(workspace) = paths
            .iter()
            .find(|path| path.is_dir() && path.join("manifest.json").is_file())
        {
            parameters.set("workspace", Value::Path(workspace.clone()));
            if let Some(parent) = workspace.parent() {
                parameters.set("output", Value::Path(parent.join("translation_source")));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let workspace = std::path::absolute(parameters.path("workspace")?)?;
        let output = std::path::absolute(parameters.path("output")?)?;
        let prepared =
            prepare_translation_export(&workspace, &output, parameters.flag("overwrite"))
                .map_err(vn_cli::Error)?;
        let preview = Preview {
            inputs: vec![workspace],
            outputs: vec![output],
            steps: vec![
                "核对解包清单、原盘哈希和八个脚本哈希".into(),
                "按 N88 token 边界识别 A$/说话人片段/PRINT 字符串".into(),
                "保留脚本控制布局，只导出全角化的有效正文".into(),
            ],
            details: vec![
                format!("脚本：{} 个", prepared.scripts()),
                format!("可编辑条目：{} 个", prepared.entries()),
            ],
        };
        Ok(Box::new(ExportJob { preview, prepared }))
    }
}

struct ExportJob {
    preview: Preview,
    prepared: PreparedTranslationExport,
}

impl PreparedOperation for ExportJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在写出正式翻译 JSON")?;
        let report = self.prepared.write().map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "八个脚本的正式翻译 JSON 已导出；只编辑 message 字段。".into(),
            totals: vec![
                ("脚本数".into(), report.scripts as u64),
                ("文本条目".into(), report.entries as u64),
            ],
            outputs: vec![report.output_root],
            warnings: Vec::new(),
        })
    }
}

struct BuildLocalized;

impl Operation for BuildLocalized {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "build",
            "注入译文并生成英文 D88",
            vec![
                Field::new("source", "原始 軽井沢 D88", FieldKind::Path).required(),
                Field::new(
                    "translation",
                    "翻译 JSON 文件或目录（可多选）",
                    FieldKind::Paths,
                )
                .required(),
                Field::new("font-rom", "实验确认的 FONT.ROM", FieldKind::Path).required(),
                Field::new("output", "独立成品输出目录", FieldKind::Path).required(),
                Field::new(
                    "allow-incomplete",
                    "允许未翻译日文（仅阶段测试）",
                    FieldKind::Flag,
                )
                .default(Value::Flag(false)),
                Field::new(
                    "large-font",
                    "16x12 大字及正文分页（诊断）",
                    FieldKind::Flag,
                )
                .default(Value::Flag(false)),
                Field::new(
                    "no-compression",
                    "禁用运行时文本字典（仅诊断）",
                    FieldKind::Flag,
                )
                .default(Value::Flag(false)),
            ],
        );
        spec.description = "校验译文与源 BASIC，默认生成 8x8 英文字形；--large-font 使用 16x12 字形、正文换行分页和紧凑菜单。文本字典随启动盘载入到 F200 保留区，避免后续章节覆盖。按原 N88 目录/FAT 写回独立 D88，并读回验证。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(source) = paths.iter().find(|path| has_extension(path, "d88")) {
            parameters.set("source", Value::Path(source.clone()));
            if let Some(parent) = source.parent() {
                let font = parent.join("font.rom");
                if font.is_file() {
                    parameters.set("font-rom", Value::Path(font));
                }
                parameters.set("output", Value::Path(parent.join("work").join("build")));
            }
        }
        let translations = paths
            .iter()
            .filter(|path| {
                path.is_dir()
                    || path
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case("json"))
            })
            .cloned()
            .collect::<Vec<_>>();
        if !translations.is_empty() {
            parameters.set("translation", Value::Paths(translations));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = std::path::absolute(parameters.path("source")?)?;
        let translations = parameters.paths("translation")?.to_vec();
        let font_rom = std::path::absolute(parameters.path("font-rom")?)?;
        let output = std::path::absolute(parameters.path("output")?)?;
        let allow_incomplete = parameters.flag("allow-incomplete");
        let prepared = prepare_localized_build(
            &source,
            &translations,
            &font_rom,
            &output,
            karuizawa_d88_tool::workflow::BuildOptions {
                overwrite: parameters.flag("overwrite"),
                allow_incomplete,
                no_compression: parameters.flag("no-compression"),
                large_font: parameters.flag("large-font"),
            },
        )
        .map_err(vn_cli::Error)?;
        let mut inputs = vec![source, font_rom];
        inputs.extend(translations);
        let preview = Preview {
            inputs,
            outputs: vec![output],
            steps: vec![
                "校验源盘、FONT.ROM、JSON 原文与控制标签".into(),
                if parameters.flag("large-font") {
                    "重建 N88-BASIC，使用 16x12 大字并重新分页"
                } else {
                    "重建变长 N88-BASIC，并把绘制宽度改为 8px"
                }
                .into(),
                "用 FONT.ROM 生成完整半角 ASCII 字形表".into(),
                "追加必要空闲簇，写回 FAT 与原扇区，保留保护轨布局".into(),
                "重新解析成品 D88/N88 并逐文件读回核对".into(),
            ],
            details: vec![
                format!("脚本：{} 个", prepared.scripts()),
                format!("条目：{} 个", prepared.entries()),
                format!("修改条目：{} 个", prepared.changed_entries()),
                format!("未翻译日文：{} 条", prepared.untranslated_entries()),
                format!("新增 FAT 簇：{} 个", prepared.allocated_new_clusters()),
            ],
        };
        Ok(Box::new(BuildJob { preview, prepared }))
    }
}

struct BuildJob {
    preview: Preview,
    prepared: PreparedLocalizedBuild,
}

impl PreparedOperation for BuildJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在提交英文 D88、补丁文件和构建报告")?;
        let report = self.prepared.write().map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "英文 D88 已生成，并已通过 D88/N88 结构读回。".into(),
            totals: vec![
                ("脚本数".into(), report.scripts as u64),
                ("文本条目".into(), report.entries as u64),
                ("修改条目".into(), report.changed_entries as u64),
                ("修改文件".into(), report.changed_files as u64),
                ("新增 FAT 簇".into(), report.allocated_new_clusters as u64),
            ],
            outputs: vec![report.output_root, report.output_d88],
            warnings: report.warnings,
        })
    }
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "《軽井沢シンドローム》PC-8801 英文化工具",
            env!("CARGO_PKG_VERSION"),
            vec![Box::new(ExportTranslation), Box::new(BuildLocalized)],
        )?
        .run_env()
    })();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[失败] {error}");
            ExitCode::FAILURE
        }
    }
}
