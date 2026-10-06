//! `vn-cli` operations implemented by this game adapter.

use std::collections::BTreeMap;
use std::path::PathBuf;

use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Parameters, PreparedOperation, Preview, Progress,
    RunReport, Value,
};

use crate::output::{write_directory_transaction, DirectorySnapshot};
use crate::program::{
    extract_main_program, rebuild_main_program, MainProgramExtraction, MainProgramRebuild,
    MAIN_PROGRAM_BASE,
};
use crate::text::{parse_text_source, TextBuild, TextPrimaryManifest};

pub struct ExportText;

pub struct ImportText;

pub struct ExtractMain;

pub struct RebuildMain;

impl Operation for ExtractMain {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "extract-main",
            "提取主程序映像",
            vec![
                Field::new("source", "原始启动软碟 D88", FieldKind::Path).required(),
                Field::new("output", "独立输出目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description =
            "依照 IDA Z80 确认的 C200 loader 表，从 D88 重建 0x0100 主程序映像及来源映射。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(source) = paths.first() {
            parameters.set("source", Value::Path(source.clone()));
            if let Some(root) = source.parent() {
                parameters.set(
                    "output",
                    Value::Path(root.join("work/analysis/main_program")),
                );
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source = parameters.path("source")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        let source_bytes = std::fs::read(&source)
            .map_err(|error| format!("无法读取 {}：{error}", source.display()))?;
        let extraction = extract_main_program(&source_bytes).map_err(vn_cli::Error::from)?;
        let output_snapshot = DirectorySnapshot::capture(&output).map_err(vn_cli::Error::from)?;
        let files = extraction_files(&extraction)?;
        let preview = Preview {
            inputs: vec![source],
            outputs: vec![output.clone()],
            steps: vec![
                "从 A 盘 C0/H0/R5-R7 拼合 C200 loader".into(),
                "解析 loader 六组三字节装载表并核验 D88 CHR 扇区".into(),
                "提取 0x0100 起始的六个 4 KiB 主程序块".into(),
            ],
            details: vec![
                format!(
                    "输入 SHA-256: {}；loader SHA-256: {}。",
                    extraction.report.source_sha256, extraction.report.loader_sha256
                ),
                format!(
                    "主程序 {} 字节，入口 0x{:04X}，SHA-256: {}。",
                    extraction.program.len(),
                    extraction.report.program_base,
                    extraction.report.program_sha256
                ),
                format!("输出目录：{}。", output.display()),
            ],
        };
        Ok(Box::new(ExtractMainJob {
            output,
            files,
            output_snapshot,
            overwrite: parameters.flag("overwrite"),
            preview,
            report: extraction.report,
        }))
    }
}

impl Operation for RebuildMain {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "rebuild-main",
            "重建主程序软碟",
            vec![
                Field::new("source", "原始启动软碟 D88", FieldKind::Path).required(),
                Field::new("program", "修改后的 0x0100 主程序映像", FieldKind::Path).required(),
                Field::new(
                    "mapping",
                    "对应原盘生成的主程序提取映射 JSON",
                    FieldKind::Path,
                )
                .required(),
                Field::new("output", "独立输出目录", FieldKind::Path).required(),
            ],
        );
        spec.primary = false;
        spec.description = "核对主程序提取映射确实对应当前原盘后，按 C200 loader 表回写 24 KiB 主程序；保持每个装载扇区 256 字节并重提取校验。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(source) = paths.first() {
            parameters.set("source", Value::Path(source.clone()));
            if let Some(root) = source.parent() {
                parameters.set(
                    "program",
                    Value::Path(root.join("work/analysis/main_program/main_program_0100.bin")),
                );
                parameters.set(
                    "mapping",
                    Value::Path(root.join("work/analysis/main_program/main_program_map.json")),
                );
                parameters.set("output", Value::Path(root.join("work/rebuilt_main")));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source = parameters.path("source")?.to_path_buf();
        let program_path = parameters.path("program")?.to_path_buf();
        let mapping_path = parameters.path("mapping")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        let source_bytes = std::fs::read(&source)
            .map_err(|error| format!("无法读取 {}：{error}", source.display()))?;
        let program_bytes = std::fs::read(&program_path)
            .map_err(|error| format!("无法读取 {}：{error}", program_path.display()))?;
        let mapping_bytes = std::fs::read(&mapping_path)
            .map_err(|error| format!("无法读取 {}：{error}", mapping_path.display()))?;
        crate::program::verify_main_program_map(&source_bytes, &mapping_bytes)
            .map_err(vn_cli::Error::from)?;
        let rebuild =
            rebuild_main_program(&source_bytes, &program_bytes).map_err(vn_cli::Error::from)?;
        let output_snapshot = DirectorySnapshot::capture(&output).map_err(vn_cli::Error::from)?;
        let files = rebuild_files(&rebuild)?;
        let preview = Preview {
            inputs: vec![source, program_path, mapping_path],
            outputs: vec![output.clone()],
            steps: vec![
                "重新提取当前 D88 的映射并与提供的来源 JSON 完整比对".into(),
                "按已匹配的 C200 loader 表核对六个主程序块来源".into(),
                "只替换内容有变化的 256 字节程序扇区，不改变扇区长度".into(),
                "重新解析 D88 并提取主程序，逐字节核对目标映像".into(),
            ],
            details: vec![
                format!(
                    "原主程序 SHA-256: {}；目标主程序 SHA-256: {}。",
                    rebuild.report.source_program_sha256, rebuild.report.rebuilt_program_sha256
                ),
                format!(
                    "主程序 {} 字节，修改 {} 个物理扇区；D88 SHA-256: {}。",
                    rebuild.report.program_size,
                    rebuild.report.changed_sectors,
                    rebuild.report.disk.rebuilt_sha256
                ),
                format!("输出目录：{}。", output.display()),
            ],
        };
        Ok(Box::new(RebuildMainJob {
            output,
            files,
            output_snapshot,
            overwrite: parameters.flag("overwrite"),
            preview,
            report: rebuild.report,
        }))
    }
}

struct ExtractMainJob {
    output: PathBuf,
    files: BTreeMap<PathBuf, Vec<u8>>,
    output_snapshot: DirectorySnapshot,
    overwrite: bool,
    preview: Preview,
    report: crate::program::MainProgramReport,
}

impl PreparedOperation for ExtractMainJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("装载器、块映射和主程序数据均已在预检中固定")?;
        progress.report(&format!("写入 {} 个文件到暂存目录", self.files.len()))?;
        write_directory_transaction(
            &self.output,
            &self.files,
            self.overwrite,
            &self.output_snapshot,
        )
        .map_err(vn_cli::Error::from)?;
        Ok(RunReport {
            summary: format!(
                "主程序提取完成：{} 字节，SHA-256 {}。",
                self.report.program_size, self.report.program_sha256
            ),
            totals: vec![
                (
                    "loader 扇区".into(),
                    self.report.loader_sectors.len() as u64,
                ),
                ("主程序块".into(), self.report.blocks.len() as u64),
                ("输出文件".into(), self.files.len() as u64),
            ],
            outputs: vec![self.output],
            warnings: Vec::new(),
        })
    }
}

struct RebuildMainJob {
    output: PathBuf,
    files: BTreeMap<PathBuf, Vec<u8>>,
    output_snapshot: DirectorySnapshot,
    overwrite: bool,
    preview: Preview,
    report: crate::program::MainProgramRebuildReport,
}

impl PreparedOperation for RebuildMainJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("D88 重建和主程序重提取比对均已在预检中完成")?;
        progress.report(&format!("写入 {} 个文件到暂存目录", self.files.len()))?;
        write_directory_transaction(
            &self.output,
            &self.files,
            self.overwrite,
            &self.output_snapshot,
        )
        .map_err(vn_cli::Error::from)?;
        Ok(RunReport {
            summary: format!(
                "主程序软碟重建完成：{} 个扇区已修改，D88 SHA-256 {}。",
                self.report.changed_sectors, self.report.disk.rebuilt_sha256
            ),
            totals: vec![
                ("修改扇区".into(), self.report.changed_sectors as u64),
                ("D88 磁盘".into(), self.report.disk.rebuilt_disks as u64),
                ("输出文件".into(), self.files.len() as u64),
            ],
            outputs: vec![self.output],
            warnings: Vec::new(),
        })
    }
}

fn extraction_files(
    extraction: &MainProgramExtraction,
) -> vn_cli::Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    files.insert(
        PathBuf::from("loader_c200_r5-r7.bin"),
        extraction.loader.clone(),
    );
    files.insert(
        PathBuf::from("main_program_0100.bin"),
        extraction.program.clone(),
    );
    for block in &extraction.report.blocks {
        let offset = usize::from(
            block
                .load_address
                .checked_sub(MAIN_PROGRAM_BASE)
                .ok_or_else(|| vn_cli::Error::from("主程序块基址低于整体映像基址"))?,
        );
        let end = offset
            .checked_add(block.loaded_bytes)
            .ok_or_else(|| vn_cli::Error::from("主程序块输出范围溢出"))?;
        let bytes = extraction
            .program
            .get(offset..end)
            .ok_or_else(|| vn_cli::Error::from("主程序块范围超出整体映像"))?;
        files.insert(
            PathBuf::from(format!(
                "main_block_{:02}_{:04X}_slot{:02}.bin",
                block.index, block.load_address, block.track_slot
            )),
            bytes.to_vec(),
        );
    }
    let report = serde_json::to_vec_pretty(&extraction.report)
        .map_err(|error| vn_cli::Error::from(format!("序列化提取映射失败：{error}")))?;
    files.insert(PathBuf::from("main_program_map.json"), report);
    Ok(files)
}

fn rebuild_files(rebuild: &MainProgramRebuild) -> vn_cli::Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    files.insert(PathBuf::from("rebuilt.d88"), rebuild.disk.clone());
    let report = serde_json::to_vec_pretty(&rebuild.report)
        .map_err(|error| vn_cli::Error::from(format!("序列化重建报告失败：{error}")))?;
    files.insert(PathBuf::from("rebuild_main_report.json"), report);
    Ok(files)
}

impl Operation for ExportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "export-text",
            "提取展望台文本",
            vec![
                Field::new("source", "原始启动软碟 D88", FieldKind::Path).required(),
                Field::new("output", "文本输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按 IDA 确认的 C000/D300/E900 文本 reader 和 mode-2 descriptor 导出原生记录及 vn-text 翻译 JSON。".into();
        spec.primary = true;
        spec.writes = true;
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(source) = paths.first() {
            parameters.set("source", Value::Path(source.clone()));
            if let Some(root) = source.parent() {
                parameters.set("output", Value::Path(root.join("work/text_export")));
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source = parameters.path("source")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        let source_bytes = std::fs::read(&source)
            .map_err(|error| format!("无法读取 {}：{error}", source.display()))?;
        let extraction = parse_text_source(&source_bytes).map_err(vn_cli::Error::from)?;
        let primary = serde_json::to_vec_pretty(&extraction.primary)
            .map_err(|error| vn_cli::Error::from(format!("序列化 primary.json 失败：{error}")))?;
        let mut primary = primary;
        primary.push(b'\n');
        let translation = vn_text::write_template(extraction.translation_entries())
            .map_err(|error| vn_cli::Error::from(format!("写入翻译 JSON 失败：{error}")))?;
        let files = BTreeMap::from([
            (PathBuf::from("primary.json"), primary),
            (PathBuf::from("translation.json"), translation),
        ]);
        let output_snapshot = DirectorySnapshot::capture(&output).map_err(vn_cli::Error::from)?;
        let records = extraction.entries.len();
        let details = extraction
            .primary
            .resources
            .iter()
            .map(|resource| {
                format!(
                    "selector {} / {:?}: {} 条记录，{} 字节，SHA-256 {}。",
                    resource.selector,
                    resource.reader,
                    resource.records.len(),
                    resource.capacity,
                    resource.source_sha256
                )
            })
            .collect();
        let preview = Preview {
            inputs: vec![source],
            outputs: vec![output.clone()],
            steps: vec![
                "从当前 D88 重建 0x0100 主程序及其单字节 JIS 映射表".into(),
                "读取实际加载的 mode-2 selector 0/1/5 descriptor".into(),
                "按 C000、D300 与 E900 reader 的 token 边界解码 NUL 记录".into(),
                "写出来源绑定的 primary.json 和 vn-text translation.json".into(),
            ],
            details,
        };
        Ok(Box::new(ExportTextJob {
            output,
            files,
            output_snapshot,
            overwrite: parameters.flag("overwrite"),
            preview,
            records,
        }))
    }
}

struct ExportTextJob {
    output: PathBuf,
    files: BTreeMap<PathBuf, Vec<u8>>,
    output_snapshot: DirectorySnapshot,
    overwrite: bool,
    preview: Preview,
    records: usize,
}

impl PreparedOperation for ExportTextJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("文本 reader、mode-2 边界和记录映射已在预检中固定")?;
        write_directory_transaction(
            &self.output,
            &self.files,
            self.overwrite,
            &self.output_snapshot,
        )
        .map_err(vn_cli::Error::from)?;
        Ok(RunReport {
            summary: format!("文本提取完成：{} 条记录。", self.records),
            totals: vec![
                ("文本记录".into(), self.records as u64),
                ("输出文件".into(), self.files.len() as u64),
            ],
            outputs: vec![self.output],
            warnings: Vec::new(),
        })
    }
}

impl Operation for ImportText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "import-text",
            "注回文本并重建软碟/字库",
            vec![
                Field::new("source", "原始启动软碟 D88", FieldKind::Path).required(),
                Field::new("primary", "对应原盘的 primary.json", FieldKind::Path).required(),
                Field::new("translation", "编辑后的翻译 JSON", FieldKind::Path).required(),
                Field::new("kanji1", "源 KANJI1.ROM", FieldKind::Path).required(),
                Field::new("font-face", "缺失字形绘制字体", FieldKind::Text).default(Value::Text(
                    vn_font::font_88::DEFAULT_GLYPH_FONT_FACE.into(),
                )),
                Field::new("glyphs", "可选的已有 FCG1 缺字点阵表", FieldKind::Path),
                Field::new("output", "重建输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "校验原盘与 primary.json 完全匹配，按文本 token 规则注回翻译；容量不足时把高频汉字压成空闲单字节 token 并同步改主程序映射表。输出的 rebuilt.d88 与同目录 kanji1.rom 必须配套使用；可先载入已有 FCG1 点阵，再由 Windows 字体补画剩余缺字。".into();
        spec.primary = true;
        spec.writes = true;
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> vn_cli::Result<()> {
        if let Some(source) = paths.first() {
            parameters.set("source", Value::Path(source.clone()));
            if let Some(root) = source.parent() {
                parameters.set(
                    "primary",
                    Value::Path(root.join("work/text_export/primary.json")),
                );
                parameters.set(
                    "translation",
                    Value::Path(root.join("work/text_export/translation.json")),
                );
                parameters.set("output", Value::Path(root.join("work/text_import")));
                if let Some(games_root) = root.parent() {
                    parameters.set(
                        "kanji1",
                        Value::Path(games_root.join("M88_212a/kanji1.rom")),
                    );
                }
            }
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> vn_cli::Result<Box<dyn PreparedOperation>> {
        let source = parameters.path("source")?.to_path_buf();
        let primary_path = parameters.path("primary")?.to_path_buf();
        let translation_path = parameters.path("translation")?.to_path_buf();
        let kanji1_path = parameters.path("kanji1")?.to_path_buf();
        let font_face = parameters.text("font-face")?.to_owned();
        let glyph_table_path = match parameters.get("glyphs") {
            Some(Value::Path(path)) => Some(path.to_path_buf()),
            _ => None,
        };
        let output = parameters.path("output")?.to_path_buf();
        let source_bytes = std::fs::read(&source)
            .map_err(|error| format!("无法读取 {}：{error}", source.display()))?;
        let primary_bytes = std::fs::read(&primary_path)
            .map_err(|error| format!("无法读取 {}：{error}", primary_path.display()))?;
        let translation_bytes = std::fs::read(&translation_path)
            .map_err(|error| format!("无法读取 {}：{error}", translation_path.display()))?;
        let kanji1_rom = std::fs::read(&kanji1_path)
            .map_err(|error| format!("无法读取 {}：{error}", kanji1_path.display()))?;
        let glyph_table = if let Some(path) = glyph_table_path.as_ref() {
            Some(
                std::fs::read(path)
                    .map_err(|error| format!("无法读取 FCG1 点阵表 {}：{error}", path.display()))?,
            )
        } else {
            None
        };

        let extraction = parse_text_source(&source_bytes).map_err(vn_cli::Error::from)?;
        let supplied_primary: TextPrimaryManifest = serde_json::from_slice(&primary_bytes)
            .map_err(|error| vn_cli::Error::from(format!("primary.json 格式无效：{error}")))?;
        if supplied_primary != extraction.primary {
            return Err(
                "primary.json 与当前 D88 的哈希、selector 边界或记录顺序不匹配；请重新导出".into(),
            );
        }
        let translations = vn_text::read_template(&translation_bytes)
            .map_err(|error| vn_cli::Error::from(format!("翻译 JSON 格式无效：{error}")))?;
        vn_text::validate_shape(extraction.translation_entries(), &translations)
            .map_err(|error| vn_cli::Error::from(format!("翻译 JSON 与当前来源不匹配：{error}")))?;
        let build = extraction
            .rebuild_translations_with_font_face_and_glyph_table(
                &source_bytes,
                &translations,
                &kanji1_rom,
                &font_face,
                glyph_table.as_deref(),
            )
            .map_err(vn_cli::Error::from)?;
        let files = text_build_files(&build, &translation_bytes)?;
        let output_snapshot = DirectorySnapshot::capture(&output).map_err(vn_cli::Error::from)?;
        let preview = Preview {
            inputs: {
                let mut inputs = vec![source, primary_path, translation_path, kanji1_path];
                if let Some(path) = glyph_table_path {
                    inputs.push(path);
                }
                inputs
            },
            outputs: vec![output.clone()],
            steps: vec![
                "从原始 D88 重新解析文本并与 primary.json 完整比较".into(),
                "校验 vn-text 条目数与顺序；容量不足时规划高频汉字单字节别名".into(),
                format!("复用可选 FCG1 点阵，并用 Windows 字体 {font_face} 绘制其余缺字"),
                "只改写 selector 0/1/5 原有 descriptor 区间，保留原容量及索引".into(),
                "重建同目录 KANJI1.ROM；它必须与 rebuilt.d88 配套使用".into(),
                "重新解析 D88 并核对所有输出文本、tag 和索引".into(),
            ],
            details: vec![
                format!(
                    "{} 条记录，{} 条正文被改写；selector 容量：{:?}；单字节映射压缩 {} 字节。",
                    build.report.total_records,
                    build.report.changed_records,
                    build.report.resource_capacities,
                    build.report.compression_saved_bytes
                ),
                format!(
                    "现场生成 {} 个缺失字形。",
                    build.font.manifest.generated_glyphs.len()
                ),
                format!(
                    "D88 SHA-256: {}；KANJI1.ROM 字节数：{}。",
                    build.report.rebuilt_d88_sha256,
                    build.kanji1_rom.len()
                ),
                format!("输出目录：{}。", output.display()),
            ],
        };
        Ok(Box::new(ImportTextJob {
            output,
            files,
            output_snapshot,
            overwrite: parameters.flag("overwrite"),
            preview,
            report: build.report,
        }))
    }
}

fn text_build_files(
    build: &TextBuild,
    translation_bytes: &[u8],
) -> vn_cli::Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    files.insert(PathBuf::from("rebuilt.d88"), build.disk.clone());
    files.insert(PathBuf::from("kanji1.rom"), build.kanji1_rom.clone());
    if let Some(glyphs) = &build.generated_glyph_table {
        files.insert(PathBuf::from("generated_glyphs.fcg1"), glyphs.clone());
    }
    files.insert(
        PathBuf::from("translation.json"),
        translation_bytes.to_vec(),
    );
    let font = serde_json::to_vec_pretty(&build.font)
        .map_err(|error| vn_cli::Error::from(format!("序列化字库报告失败：{error}")))?;
    let mut font = font;
    font.push(b'\n');
    files.insert(PathBuf::from("font_build_report.json"), font);
    let report = serde_json::to_vec_pretty(&build.report)
        .map_err(|error| vn_cli::Error::from(format!("序列化文本回注报告失败：{error}")))?;
    let mut report = report;
    report.push(b'\n');
    files.insert(PathBuf::from("text_import_report.json"), report);
    Ok(files)
}

struct ImportTextJob {
    output: PathBuf,
    files: BTreeMap<PathBuf, Vec<u8>>,
    output_snapshot: DirectorySnapshot,
    overwrite: bool,
    preview: Preview,
    report: crate::text::TextBuildReport,
}

impl PreparedOperation for ImportTextJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> vn_cli::Result<RunReport> {
        progress.report("原盘身份、翻译结构、token 编码、D88 重建和回读校验均已在预检中完成")?;
        write_directory_transaction(
            &self.output,
            &self.files,
            self.overwrite,
            &self.output_snapshot,
        )
        .map_err(vn_cli::Error::from)?;
        Ok(RunReport {
            summary: format!(
                "文本注回完成：{} 条记录已更新 {} 条；D88 SHA-256 {}。",
                self.report.total_records,
                self.report.changed_records,
                self.report.rebuilt_d88_sha256
            ),
            totals: vec![
                ("文本记录".into(), self.report.total_records as u64),
                ("正文已改写".into(), self.report.changed_records as u64),
                (
                    "修改 D88 扇区".into(),
                    self.report.disk_rebuild.changed_sectors as u64,
                ),
                ("输出文件".into(), self.files.len() as u64),
            ],
            outputs: vec![self.output],
            warnings: Vec::new(),
        })
    }
}
