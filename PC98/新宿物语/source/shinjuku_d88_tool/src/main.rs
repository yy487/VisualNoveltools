use shinjuku_d88_tool::workflow::{
    prepare_localized_build, prepare_translation_export, PreparedLocalizedBuild,
    PreparedTranslationExport, FONT_FACE,
};
use shinjuku_d88_tool::{prepare_batch, validate_output_target, write_prepared, PreparedBatch};
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
            "导出 MES 翻译 JSON",
            vec![
                Field::new("workspace", "已有解包工作区", FieldKind::Path).required(),
                Field::new("output", "翻译 JSON 输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "从已解包的具体 FAT12 文件中读取 MES；不重复解包 D88。每个源脚本对应一个 UTF-8 JSON。"
                .into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths
            .iter()
            .find(|path| path.is_dir() && path.join("workspace.json").is_file())
        {
            parameters.set("workspace", Value::Path(path.clone()));
            parameters.set(
                "output",
                Value::Path(path.parent().unwrap_or(Path::new(".")).join("translation")),
            );
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
                "核对 workspace.json 与已解包 MES 哈希".into(),
                "按已确认的 ADV98 字节码长度标记正文与操作数".into(),
                "写出一脚本一 JSON；A5 等控制结构保持不可编辑".into(),
            ],
            details: vec![
                format!("脚本: {} 个", prepared.scripts()),
                format!("可翻译条目: {} 个", prepared.entries()),
                format!("词法警告: {} 个", prepared.warnings()),
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
        progress.report("正在写出 MES 翻译工作区")?;
        let report = self.prepared.write().map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "MES 翻译 JSON 已导出；scr_msg/_scr_name 与脚本控制结构受保护。".into(),
            totals: vec![
                ("脚本数".into(), report.scripts as u64),
                ("文本条目".into(), report.entries as u64),
            ],
            outputs: vec![report.output_root],
            warnings: report.warnings,
        })
    }
}

struct BuildLocalized;

impl Operation for BuildLocalized {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "build",
            "生成汉化 D88 与 font.bmp",
            vec![
                Field::new("d88", "原始 D88 文件或目录（可多选）", FieldKind::Paths).required(),
                Field::new("workspace", "对应的已有解包工作区", FieldKind::Path).required(),
                Field::new("translation", "已编辑的翻译 JSON 目录", FieldKind::Path).required(),
                Field::new("output", "独立成品输出目录", FieldKind::Path).required(),
                Field::new("font-face", "中文字形字体", FieldKind::Text)
                    .default(Value::Text(FONT_FACE.into())),
            ],
        );
        spec.description = "校验翻译、统一规划 CP932 字槽、重建 MES/FAT12/D88，并直接输出 NP2/NP2kai 可用的 2048×2048 1bpp font.bmp。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        let d88 = paths
            .iter()
            .filter(|path| {
                path.is_dir()
                    || path
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case("d88"))
            })
            .cloned()
            .collect::<Vec<_>>();
        if !d88.is_empty() {
            parameters.set("d88", Value::Paths(d88));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let d88 = parameters.paths("d88")?.to_vec();
        let workspace = std::path::absolute(parameters.path("workspace")?)?;
        let translation = std::path::absolute(parameters.path("translation")?)?;
        let output = std::path::absolute(parameters.path("output")?)?;
        let face = parameters.text("font-face")?.to_owned();
        let prepared = prepare_localized_build(
            &d88,
            &workspace,
            &translation,
            &output,
            parameters.flag("overwrite"),
            &face,
        )
        .map_err(vn_cli::Error)?;
        let preview = Preview {
            inputs: d88
                .iter()
                .cloned()
                .chain([workspace, translation])
                .collect(),
            outputs: vec![output],
            steps: vec![
                "校验 JSON 的源文、偏移、token IR 与源 MES".into(),
                "全局规划双字节 CP932 载体并重绘 16×16 字形".into(),
                "重建变长 MES，重新分配 FAT12 簇链".into(),
                "写回原 D88 扇区位置并复查成品文件哈希".into(),
            ],
            details: vec![
                format!("D88: {} 张", prepared.disks()),
                format!(
                    "MES: {} 个 / 条目 {} 个",
                    prepared.scripts(),
                    prepared.entries()
                ),
                format!("修改条目: {} 个", prepared.changed_entries()),
                format!("重绘字形: {} 个", prepared.patched_glyphs()),
                "成品字库文件名: font.bmp".into(),
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
        progress.report("正在提交汉化 D88、font.bmp 与映射清单")?;
        let report = self.prepared.write().map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "汉化成品已生成：D88 位于 d88 子目录，font.bmp 位于成品根目录。".into(),
            totals: vec![
                ("D88".into(), report.disks as u64),
                ("MES".into(), report.scripts as u64),
                ("文本条目".into(), report.entries as u64),
                ("修改条目".into(), report.changed_entries as u64),
                ("修改脚本".into(), report.changed_scripts as u64),
                ("重绘字形".into(), report.patched_glyphs as u64),
            ],
            outputs: vec![report.output_root],
            warnings: report.warnings,
        })
    }
}

struct Unpack;

impl Operation for Unpack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解包 D88 游戏资源",
            vec![
                Field::new(
                    "input",
                    "D88 文件或包含 D88 的目录（可多选）",
                    FieldKind::Paths,
                )
                .required(),
                Field::new("output", "资源输出目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description = "解析 D88 和 FAT12，导出具体活动文件；不会输出轨道/扇区切片。已有解包工作区无需再次执行。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if paths.is_empty() {
            return Ok(());
        }
        parameters.set("input", Value::Paths(paths.to_vec()));
        if let Some(first) = paths.first() {
            let parent = if first.is_dir() {
                first.as_path()
            } else {
                first.parent().unwrap_or(Path::new("."))
            };
            parameters.set("output", Value::Path(parent.join("unpack")));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let output = std::path::absolute(parameters.path("output")?)?;
        let overwrite = parameters.flag("overwrite");
        validate_output_target(&output, overwrite).map_err(vn_cli::Error)?;
        let prepared = prepare_batch(parameters.paths("input")?).map_err(vn_cli::Error)?;
        let summary = prepared.summary();
        let details = prepared
            .disk_summaries()
            .into_iter()
            .map(|disk| {
                format!(
                    "{}: {} 文件；{} 轨 / {} 扇区",
                    disk.source_file, disk.files, disk.tracks, disk.sectors
                )
            })
            .collect();
        Ok(Box::new(UnpackJob {
            preview: Preview {
                inputs: prepared.input_paths().to_vec(),
                outputs: vec![output.clone()],
                steps: vec![
                    "验证 D88 轨道、扇区和容器声明大小".into(),
                    "验证 FAT12 目录项、簇链和文件边界".into(),
                    "事务式写出活动文件树与 workspace.json".into(),
                ],
                details,
            },
            prepared,
            output,
            overwrite,
            summary,
        }))
    }
}

struct UnpackJob {
    preview: Preview,
    prepared: PreparedBatch,
    output: PathBuf,
    overwrite: bool,
    summary: shinjuku_d88_tool::PreparationSummary,
}

impl PreparedOperation for UnpackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在写出 FAT12 活动文件树")?;
        let report =
            write_prepared(self.prepared, &self.output, self.overwrite).map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "D88 游戏资源解包完成；输出是 FAT12 具体文件。".into(),
            totals: vec![
                ("磁盘数".into(), self.summary.images as u64),
                ("文件数".into(), report.extracted_files as u64),
                ("目录数".into(), report.extracted_directories as u64),
                ("文件字节数".into(), report.extracted_bytes),
            ],
            outputs: vec![report.output_root],
            warnings: report.warnings,
        })
    }
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "《新宿物語》D88 汉化工具",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(ExportTranslation),
                Box::new(BuildLocalized),
                Box::new(Unpack),
            ],
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
