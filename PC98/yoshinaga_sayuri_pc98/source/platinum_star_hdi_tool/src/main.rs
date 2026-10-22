use platinum_star_hdi_tool::{
    prepare_hdi_pack, prepare_text_export, prepare_text_import, prepare_unpack, HdiPackReport,
    PreparedHdiPack, PreparedTextExport, PreparedTextImport, PreparedUnpack,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation, Preview,
    Progress, Result, RunReport, Value,
};

struct UnpackHdi;

impl Operation for UnpackHdi {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解包 HDI 硬盘镜像",
            vec![
                Field::new("source", "只读源 HDI", FieldKind::Path).required(),
                Field::new("output", "独立解包目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "校验 Anex86 HDI、PC-98 分区表、FAT12 BPB、目录树和簇链，事务式提取活动文件并生成结构清单。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(source) = paths.first() {
            parameters.set("source", Value::Path(source.clone()));
            parameters.set("output", Value::Path(default_output(source)?));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = parameters.path("source")?;
        let output = std::path::absolute(parameters.path("output")?)?;
        let prepared =
            prepare_unpack(source, &output, parameters.flag("overwrite")).map_err(vn_cli::Error)?;
        let info = prepared.inspection();
        Ok(Box::new(UnpackJob {
            preview: Preview {
                inputs: vec![prepared.source().to_path_buf()],
                outputs: vec![prepared.output().to_path_buf()],
                steps: vec![
                    "验证 HDI 几何和 PC-98 分区表".into(),
                    "验证 FAT12、目录项、簇链、循环和交叉占用".into(),
                    "写入临时目录并逐文件哈希复核后提交".into(),
                ],
                details: vec![
                    format!(
                        "源镜像: {} 字节；SHA-256 {}",
                        info.source_bytes, info.source_sha256
                    ),
                    format!(
                        "分区偏移: 0x{:X}；逻辑扇区: {} 字节；簇: {} 字节",
                        info.partition_offset, info.bytes_per_sector, info.cluster_bytes
                    ),
                    format!(
                        "活动成员: {} 个文件 / {} 个目录 / {} 字节",
                        info.files, info.directories, info.extracted_bytes
                    ),
                    format!(
                        "孤儿簇: {}；FAT 副本不一致字节: {}",
                        info.orphan_clusters, info.fat_mismatch_bytes
                    ),
                    format!(
                        "BPB/FAT 介质描述符一致: {}",
                        if info.media_descriptor_matches_fat {
                            "是"
                        } else {
                            "否（保留源盘事实并在结果中警告）"
                        }
                    ),
                ],
            },
            prepared,
        }))
    }
}

struct PackHdi;

impl Operation for PackHdi {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "pack-hdi",
            "回封 HDI 硬盘镜像",
            vec![
                Field::new("source", "只读源 HDI", FieldKind::Path).required(),
                Field::new("input", "回注后的游戏目录", FieldKind::Path).required(),
                Field::new("output", "独立新 HDI", FieldKind::Path).required(),
            ],
        );
        spec.description = "以源 HDI 的几何、分区和 FAT12 为基线，把回注目录中的活动文件重新分配到簇中，复核后写出新的 HDI。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        let Some(path) = paths.first() else {
            return Ok(());
        };
        let source = if path.is_file() {
            path.clone()
        } else {
            path.join("star_pt.hdi")
        };
        let parent = source
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        parameters.set("source", Value::Path(source));
        parameters.set("input", Value::Path(parent.join("re")));
        parameters.set("output", Value::Path(parent.join("star_pt_translated.hdi")));
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = std::path::absolute(parameters.path("source")?)?;
        let input = std::path::absolute(parameters.path("input")?)?;
        let output = std::path::absolute(parameters.path("output")?)?;
        let prepared = prepare_hdi_pack(&source, &input, &output, parameters.flag("overwrite"))
            .map_err(vn_cli::Error)?;
        let info = prepared.inspection();
        Ok(Box::new(PackHdiJob {
            preview: Preview {
                inputs: vec![
                    prepared.source().to_path_buf(),
                    prepared.input_root().to_path_buf(),
                ],
                outputs: vec![prepared.output().to_path_buf()],
                steps: vec![
                    "校验源 HDI 几何、分区、FAT12 和活动目录树".into(),
                    "按活动文件映射重新分配 FAT12 簇并更新目录项".into(),
                    "回读新 HDI，逐文件复核大小、簇链和 SHA-256 后提交".into(),
                ],
                details: vec![
                    format!("源 HDI: {} 字节", info.source_bytes),
                    format!(
                        "活动文件: {} 个；待更新: {} 个",
                        info.source_files, info.changed_files
                    ),
                    format!(
                        "写入文件字节: {}；释放后可用簇: {}",
                        info.changed_bytes, info.free_clusters
                    ),
                ],
            },
            prepared,
        }))
    }
}

struct ExtractText;

impl Operation for ExtractText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "extract-text",
            "导出翻译文本",
            vec![
                Field::new("source", "解包后的 STAR_PT 目录", FieldKind::Path).required(),
                Field::new("output", "独立 JSON 输出目录", FieldKind::Path).required(),
                Field::new("jobs", "并行任务数（1..64）", FieldKind::Text)
                    .default(Value::Text("4".into())),
            ],
        );
        spec.description = "结构化解析 SCENARIO.GDM，并解开 SPD.BIN、SPG.BIN 提取静态文本；每次发言和每个菜单项分别输出一条记录。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        let Some(path) = paths.first() else {
            return Ok(());
        };
        let source = if path.is_file() {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        } else {
            path.clone()
        };
        parameters.set("source", Value::Path(source.clone()));
        parameters.set("output", Value::Path(default_text_output(&source)?));
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = std::path::absolute(parameters.path("source")?)?;
        let output = std::path::absolute(parameters.path("output")?)?;
        let jobs = parameters
            .text("jobs")?
            .parse::<usize>()
            .map_err(|_| vn_cli::Error("jobs 必须是 1..64 的整数".into()))?;
        let prepared = prepare_text_export(&source, &output, jobs, parameters.flag("overwrite"))
            .map_err(vn_cli::Error)?;
        let info = prepared.inspection();
        Ok(Box::new(TextExportJob {
            preview: Preview {
                inputs: vec![prepared.source_root().to_path_buf()],
                outputs: vec![prepared.output().to_path_buf()],
                steps: vec![
                    "并行解包并校验 39 个 SCENARIO.GDM 场景块".into(),
                    "拆分姓名与正文，移除显示控制符并保留结构元数据".into(),
                    "解开 SPD.BIN、SPG.BIN 并按独立帮助文本/菜单项导出".into(),
                    "事务式写入三个 UTF-8 JSON 和文本清单".into(),
                ],
                details: vec![
                    format!("来源文件: {}", info.source_files),
                    format!(
                        "SCENARIO.GDM: {} 块 / {} 条",
                        info.scenario_blocks, info.scenario_entries
                    ),
                    format!("SPD.BIN: {} 条", info.spd_entries),
                    format!("SPG.BIN: {} 条", info.spg_entries),
                ],
            },
            prepared,
        }))
    }
}

struct ImportText;

impl Operation for ImportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import-text",
            "导入翻译并生成游戏目录",
            vec![
                Field::new("source", "只读 STAR_PT 来源目录", FieldKind::Path).required(),
                Field::new("translations", "翻译 JSON 目录", FieldKind::Path).required(),
                Field::new("output", "独立新游戏目录", FieldKind::Path).required(),
                Field::new("jobs", "并行任务数（1..64）", FieldKind::Text)
                    .default(Value::Text("4".into())),
            ],
        );
        spec.description = "校验三份翻译 JSON，只接受 name/message 修改，保留控制指令并重建 SCENARIO.GDM、SPD.BIN、SPG.BIN；完整复制其余游戏文件到新目录。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        let Some(path) = paths.first() else {
            return Ok(());
        };
        let source = if path.is_file() {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        } else {
            path.clone()
        };
        parameters.set("source", Value::Path(source.clone()));
        parameters.set(
            "translations",
            Value::Path(default_translation_input(&source)?),
        );
        parameters.set("output", Value::Path(default_import_output(&source)?));
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let source = std::path::absolute(parameters.path("source")?)?;
        let translations = std::path::absolute(parameters.path("translations")?)?;
        let output = std::path::absolute(parameters.path("output")?)?;
        let jobs = parameters
            .text("jobs")?
            .parse::<usize>()
            .map_err(|_| vn_cli::Error("jobs 必须是 1..64 的整数".into()))?;
        let prepared = prepare_text_import(
            &source,
            &translations,
            &output,
            jobs,
            parameters.flag("overwrite"),
        )
        .map_err(vn_cli::Error)?;
        let info = prepared.inspection();
        Ok(Box::new(TextImportJob {
            preview: Preview {
                inputs: vec![
                    prepared.source_root().to_path_buf(),
                    prepared.translation_root().to_path_buf(),
                ],
                outputs: vec![prepared.output().to_path_buf()],
                steps: vec![
                    "核对翻译 JSON 与当前三份来源文件及全部定位元数据".into(),
                    "按原指令位置回填姓名和正文，重建 GDM 与两个 MZ 文件".into(),
                    "复制完整游戏目录，在同级临时目录逐文件复核后提交".into(),
                ],
                details: vec![
                    format!("游戏文件快照: {} 个", info.source_files),
                    format!("SCENARIO.GDM: {} 条", info.scenario_entries),
                    format!("SPD.BIN: {} 条", info.spd_entries),
                    format!("SPG.BIN: {} 条", info.spg_entries),
                    format!("实际修改: {} 条", info.changed_entries),
                ],
            },
            prepared,
        }))
    }
}

struct TextExportJob {
    preview: Preview,
    prepared: PreparedTextExport,
}

struct TextImportJob {
    preview: Preview,
    prepared: PreparedTextImport,
}

struct PackHdiJob {
    preview: Preview,
    prepared: PreparedHdiPack,
}

impl PreparedOperation for PackHdiJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在回封 HDI 并复核 FAT12 文件")?;
        let report: HdiPackReport = self.prepared.execute().map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "HDI 回封完成；源 HDI 和回注目录均未修改。".into(),
            totals: vec![
                ("活动文件".into(), report.source_files as u64),
                ("修改文件".into(), report.changed_files as u64),
                ("写入文件字节".into(), report.changed_bytes),
            ],
            outputs: vec![report.output],
            warnings: report.warnings,
        })
    }
}

impl PreparedOperation for TextImportJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在生成并复核独立游戏目录")?;
        let report = self.prepared.execute().map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "文本回注完成；原始游戏目录和翻译 JSON 均未修改。".into(),
            totals: vec![
                ("复制文件".into(), report.copied_files as u64),
                ("修改文本条目".into(), report.changed_entries as u64),
            ],
            outputs: vec![report.output_root, report.manifest],
            warnings: report.warnings,
        })
    }
}

impl PreparedOperation for TextExportJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在事务式写出并复核 UTF-8 JSON")?;
        let report = self.prepared.execute().map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "文本 JSON 导出完成；原始游戏文件未修改。".into(),
            totals: vec![
                ("来源文件".into(), report.source_files as u64),
                ("场景块".into(), report.scenario_blocks as u64),
                ("文本条目".into(), report.entries as u64),
            ],
            outputs: vec![report.output_root, report.manifest],
            warnings: report.warnings,
        })
    }
}

struct UnpackJob {
    preview: Preview,
    prepared: PreparedUnpack,
}

impl PreparedOperation for UnpackJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在事务式写出解包目录并复核文件哈希")?;
        let report = self.prepared.execute().map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "HDI 解包完成；源镜像未修改。".into(),
            totals: vec![
                ("文件数".into(), report.files as u64),
                ("目录数".into(), report.directories as u64),
                ("文件内容字节".into(), report.extracted_bytes),
                ("孤儿簇".into(), report.orphan_clusters),
                ("FAT 副本不一致字节".into(), report.fat_mismatch_bytes),
            ],
            outputs: vec![report.output_root, report.manifest],
            warnings: report.warnings,
        })
    }
}

fn default_output(source: &Path) -> Result<PathBuf> {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| vn_cli::Error(format!("无法从路径取得文件名: {}", source.display())))?;
    Ok(source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}_unpacked")))
}

fn default_text_output(source: &Path) -> Result<PathBuf> {
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("platinum_star");
    Ok(source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}_translation_json")))
}

fn default_translation_input(source: &Path) -> Result<PathBuf> {
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("platinum_star");
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    for ancestor in source.ancestors().skip(1) {
        let candidate = ancestor.join("translation_json");
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }
    Ok(parent.join(format!("{name}_translation_json")))
}

fn default_import_output(source: &Path) -> Result<PathBuf> {
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("STAR_PT");
    Ok(source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}_translated")))
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "《白金之星》HDI/文本工具",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(UnpackHdi),
                Box::new(PackHdi),
                Box::new(ExtractText),
                Box::new(ImportText),
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
