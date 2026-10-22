use j_no_higeki_d88_tool::workflow::{
    prepare_localized_build, prepare_translation_export, PreparedLocalizedBuild,
    PreparedTranslationExport, FONT_FACE,
};
use misty_blue_d88_tool::{
    prepare_batch, validate_output_target, write_prepared_with_profile, PreparedBatch,
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
            "导出 JACK 翻译 JSON",
            vec![
                Field::new("workspace", "已有解包工作区", FieldKind::Path).required(),
                Field::new("output", "翻译 JSON 输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "导出 ATX 剧情文本、ADW 命令显示词和 JACK.EXE 固定 UI；索引、别名与源结构保持只读。"
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
                Value::Path(
                    path.parent()
                        .unwrap_or(Path::new("."))
                        .join("j_no_higeki_translation"),
                ),
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
                "校验解包清单、文件大小与 SHA-256".into(),
                "解密 NOT + CP932 并按 JACK 语法提取文本".into(),
                "交叉验证 FLG、MSG、FRM 编译表".into(),
                "事务式写出三个 UTF-8 翻译 JSON".into(),
            ],
            details: vec![
                format!("ATX 文本: {} 项", prepared.atx_entries()),
                format!("ADW 显示词: {} 项", prepared.adw_entries()),
                format!("EXE UI: {} 项", prepared.exe_entries()),
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
        progress.report("正在写出 JACK 翻译工作区")?;
        let report = self.prepared.write().map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "JACK 文本已导出；只需编辑 message。".into(),
            totals: vec![("文本项".into(), report.entries as u64)],
            outputs: vec![report.output_root],
            warnings: vec!["书页、标题和场景图片不属于独立文本，未导出。".into()],
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
                Field::new(
                    "translation",
                    "翻译工作区目录或单个 JSON（可多选）",
                    FieldKind::Paths,
                )
                .required(),
                Field::new("output", "独立成品输出目录", FieldKind::Path).required(),
                Field::new("font-face", "中文字形字体", FieldKind::Text)
                    .default(Value::Text(FONT_FACE.into())),
            ],
        );
        spec.description =
            "校验 JSON 后分配 PC-98 字槽，重建 ATX/ADW/EXE、MSG/FRM、ENIX-DOS FAT12 与完整 D88。"
                .into();
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
        let translations = parameters.paths("translation")?.to_vec();
        let workspace = std::path::absolute(parameters.path("workspace")?)?;
        let output = std::path::absolute(parameters.path("output")?)?;
        let prepared = prepare_localized_build(
            &d88,
            &workspace,
            &translations,
            &output,
            parameters.flag("overwrite"),
            parameters.text("font-face")?,
        )
        .map_err(vn_cli::Error)?;
        let preview = Preview {
            inputs: d88
                .into_iter()
                .chain([workspace])
                .chain(translations)
                .collect(),
            outputs: vec![output],
            steps: vec![
                "校验 JSON 源文和全部不可变结构".into(),
                "统一分配兼容字槽并生成 NP2 font.bmp".into(),
                "按变长 ATX 重建 MSG/FRM 索引".into(),
                "重新分配 ENIX-DOS FAT12 簇链并回写完整 D88".into(),
            ],
            details: vec![
                format!("D88: {} 张", prepared.disks()),
                format!(
                    "文本项: {} / 修改 {}",
                    prepared.entries(),
                    prepared.changed_entries()
                ),
                format!(
                    "修改资源: {} / 重绘字形: {}",
                    prepared.changed_resources(),
                    prepared.patched_glyphs()
                ),
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
            summary: "汉化成品已生成：完整镜像位于 d88 子目录，font.bmp 位于成品根目录。".into(),
            totals: vec![
                ("D88".into(), report.disks as u64),
                ("文本项".into(), report.entries as u64),
                ("修改文本".into(), report.changed_entries as u64),
                ("修改资源".into(), report.changed_resources as u64),
                ("重绘字形".into(), report.patched_glyphs as u64),
            ],
            outputs: vec![report.output_root],
            warnings: Vec::new(),
        })
    }
}

struct Unpack;

impl Operation for Unpack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解包《J 的悲剧》D88 游戏资源",
            vec![
                Field::new(
                    "input",
                    "D88 文件或直接包含 D88 的目录（可多选）",
                    FieldKind::Paths,
                )
                .required(),
                Field::new("output", "资源输出目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description = "复用已验证的 ENIX-DOS D88 解包器，输出实际文件树和来源清单。".into();
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
            parameters.set("output", Value::Path(parent.join("j_no_higeki_unpack")));
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
                    "{}: {} 个文件 / {} 个目录 / {} 字节",
                    disk.source_file, disk.files, disk.directories, disk.file_bytes
                )
            })
            .collect();
        Ok(Box::new(UnpackJob {
            preview: Preview {
                inputs: prepared.input_paths().to_vec(),
                outputs: vec![output.clone()],
                steps: vec![
                    "解码标准 D88 容器与 1024 字节物理扇区".into(),
                    "解析固定 ENIX-DOS FAT12 文件系统".into(),
                    "事务式写出实际文件树与校验清单".into(),
                ],
                details,
            },
            prepared,
            output,
            overwrite,
            files: summary.files,
            directories: summary.directories,
            bytes: summary.file_bytes,
            disks: summary.images,
        }))
    }
}

struct UnpackJob {
    preview: Preview,
    prepared: PreparedBatch,
    output: PathBuf,
    overwrite: bool,
    files: usize,
    directories: usize,
    bytes: u64,
    disks: usize,
}

impl PreparedOperation for UnpackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在写出 ENIX-DOS 实际游戏文件")?;
        let output = write_prepared_with_profile(
            self.prepared,
            &self.output,
            self.overwrite,
            "J no Higeki / ENIX-DOS fixed-layout FAT12",
            "《J 的悲剧》D88 游戏资源\r\n\r\n每张盘的实际 ENIX-DOS 文件位于 <盘名>/files/。\r\nmanifest.json 记录原路径、FAT12 簇链、大小和 SHA-256。\r\n后续 export/build 操作依赖这些只读来源清单。\r\n",
        )
        .map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "《J 的悲剧》游戏资源解包完成。".into(),
            totals: vec![
                ("磁盘数".into(), self.disks as u64),
                ("文件数".into(), self.files as u64),
                ("目录数".into(), self.directories as u64),
                ("文件字节数".into(), self.bytes),
            ],
            outputs: vec![output],
            warnings: Vec::new(),
        })
    }
}

fn run() -> Result<()> {
    Panel::new(
        std::env::current_exe()?.into_os_string(),
        "《J 的悲剧》D88 汉化工具",
        env!("CARGO_PKG_VERSION"),
        vec![
            Box::new(ExportTranslation),
            Box::new(BuildLocalized),
            Box::new(Unpack),
        ],
    )?
    .run_env()
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[失败] {error}");
            ExitCode::FAILURE
        }
    }
}
