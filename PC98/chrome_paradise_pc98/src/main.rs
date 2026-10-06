use chrome_paradise_pc98::{
    extract_scr_workspace,
    font_plan::{self, FontArtifactReport, FontPlan},
    inject_scr_workspace, prepare_unpack, rebuild,
};
use std::process::ExitCode;
use vn_cli::{
    Error, Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation,
    Preview, Progress, Result, RunReport, Value,
};

struct Unpack;

impl Operation for Unpack {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "unpack",
            "解包 PC-98 软碟",
            vec![
                Field::new("input", "D88 原始软碟（可重复）", FieldKind::Paths).required(),
                Field::new("output", "新建解包工作区目录", FieldKind::Path).required(),
            ],
        );
        spec.description = "校验 D88 轨道、FAT12 和文件来源后，把多张软碟解包到独立工作区。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err(Error("本阶段只创建新工作区，请改用不存在的输出目录".into()));
        }
        let inputs = parameters.paths("input")?.to_vec();
        let output = parameters.path("output")?.to_path_buf();
        let prepared = prepare_unpack(&inputs, &output).map_err(Error)?;
        let preview = Preview {
            inputs,
            outputs: vec![prepared.output().to_path_buf()],
            steps: vec![
                "读取并验证全部 D88 轨道和 CHRN".into(),
                "按 FAT12 簇链导出每张软碟的文件和 fivec 清单".into(),
                "写入工作区清单并原子提交新目录".into(),
            ],
            details: vec![format!("输入软碟: {} 张", prepared.disk_count())],
        };
        Ok(Box::new(PreparedJob { prepared, preview }))
    }
}

struct PreparedJob {
    prepared: chrome_paradise_pc98::PreparedUnpack,
    preview: Preview,
}

struct ExtractText;

impl Operation for ExtractText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "extract-text",
            "提取 SCR 文本",
            vec![
                Field::new("input", "已解包工作区目录", FieldKind::Path).required(),
                Field::new("output", "新建文本工作区目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按 CPMAIN.EXE 反汇编确认的记录、XOR 字符串和偏移生成可审阅 JSON。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err(Error("文本提取只创建新目录，请使用不存在的输出目录".into()));
        }
        let input = parameters.path("input")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        validate_directory(&input)?;
        validate_new_output(&output)?;
        Ok(Box::new(PreparedExtractText {
            input: input.clone(),
            output: output.clone(),
            preview: Preview {
                inputs: vec![input],
                outputs: vec![output],
                steps: vec![
                    "扫描已解包工作区内的 .SCR 文件".into(),
                    "解析 CPMAIN 记录、XOR 字符串和 CP932/原始字节".into(),
                    "把所有 SCR JSON 扁平写入 translations 目录并生成文本工作区清单".into(),
                ],
                details: Vec::new(),
            },
        }))
    }
}

struct PreparedExtractText {
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    preview: Preview,
}

impl PreparedOperation for PreparedExtractText {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在提取 SCR 文本")?;
        let report = extract_scr_workspace(&self.input, &self.output).map_err(Error)?;
        Ok(RunReport {
            summary: format!("已提取 {} 个 SCR", report.files),
            totals: vec![
                ("SCR".into(), report.files as u64),
                ("字节".into(), report.bytes as u64),
                ("诊断".into(), report.diagnostics as u64),
            ],
            outputs: vec![report.output],
            warnings: Vec::new(),
        })
    }
}

struct InjectText;

impl Operation for InjectText {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "inject-text",
            "注回 SCR 文本",
            vec![
                Field::new("input", "原始已解包工作区目录", FieldKind::Path).required(),
                Field::new("translations", "文本工作区目录", FieldKind::Path).required(),
                Field::new("font-plan", "可选字体计划 JSON", FieldKind::Path),
                Field::new("output", "新建注回工作区目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "校验原始 SCR SHA-256，仅按 JSON 的 message 字段重建字符串并复核结构。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err(Error("文本注回只创建新目录，请使用不存在的输出目录".into()));
        }
        let input = parameters.path("input")?.to_path_buf();
        let translations = parameters.path("translations")?.to_path_buf();
        let font_plan = match parameters.get("font-plan") {
            Some(Value::Path(path)) => Some(path.clone()),
            Some(_) => return Err(Error("--font-plan 必须是路径".into())),
            None => None,
        };
        let output = parameters.path("output")?.to_path_buf();
        validate_directory(&input)?;
        validate_directory(&translations)?;
        validate_new_output(&output)?;
        Ok(Box::new(PreparedInjectText {
            input: input.clone(),
            translations: translations.clone(),
            font_plan: font_plan.clone(),
            output: output.clone(),
            preview: Preview {
                inputs: vec![input, translations],
                outputs: vec![output],
                steps: vec![
                    "复制原始解包工作区到新目录".into(),
                    "按 SHA-256 校验每份 SCR JSON 的来源".into(),
                    "重建 XOR 字符串并复核 CPMAIN 记录".into(),
                ],
                details: Vec::new(),
            },
        }))
    }
}

struct PreparedInjectText {
    input: std::path::PathBuf,
    translations: std::path::PathBuf,
    font_plan: Option<std::path::PathBuf>,
    output: std::path::PathBuf,
    preview: Preview,
}

impl PreparedOperation for PreparedInjectText {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在注回 SCR 文本")?;
        let report = if let Some(plan_path) = &self.font_plan {
            let json = std::fs::read_to_string(plan_path).map_err(|e| Error(e.to_string()))?;
            let plan: chrome_paradise_pc98::font_plan::FontPlan =
                serde_json::from_str(&json).map_err(|e| Error(e.to_string()))?;
            chrome_paradise_pc98::inject_scr_workspace_with_font_plan(
                &self.input,
                &self.translations,
                &self.output,
                &plan,
            )
            .map_err(Error)?
        } else {
            inject_scr_workspace(&self.input, &self.translations, &self.output).map_err(Error)?
        };
        Ok(RunReport {
            summary: format!("已注回 {} 个 SCR", report.files),
            totals: vec![
                ("SCR".into(), report.files as u64),
                ("字节".into(), report.bytes as u64),
            ],
            outputs: vec![report.output],
            warnings: Vec::new(),
        })
    }
}

struct FontPlanOp;

impl Operation for FontPlanOp {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "font-plan",
            "生成字库映射计划",
            vec![
                Field::new("input", "文本工作区或 SCR JSON", FieldKind::Path).required(),
                Field::new("output", "新建字体计划 JSON", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "读取译文 JSON，保留 CPMAIN 扩展码位并生成统一的 NP2 载体字槽计划。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err(Error("字体计划只创建新文件，请使用不存在的输出路径".into()));
        }
        let input = parameters.path("input")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        if !input.exists() {
            return Err(Error(format!("输入不存在: {}", input.display())));
        }
        validate_new_output(&output)?;
        Ok(Box::new(PreparedFontPlan {
            input: input.clone(),
            output: output.clone(),
            preview: Preview {
                inputs: vec![input],
                outputs: vec![output],
                steps: vec![
                    "读取并校验所有 SCR JSON 的来源与结构诊断".into(),
                    "从 CPMAIN CODE 标记和译文收集字槽约束".into(),
                    "写入可审阅的字体计划 JSON".into(),
                ],
                details: Vec::new(),
            },
        }))
    }
}

struct PreparedFontPlan {
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    preview: Preview,
}

impl PreparedOperation for PreparedFontPlan {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在生成字体映射计划")?;
        let plan = font_plan::write_font_plan(&self.input, &self.output).map_err(Error)?;
        Ok(RunReport {
            summary: format!("已生成字体计划，覆盖 {} 份 SCR", plan.documents),
            totals: vec![
                ("SCR".into(), plan.documents as u64),
                ("字符串".into(), plan.strings as u64),
                ("字槽映射".into(), plan.mapping.len() as u64),
            ],
            outputs: vec![self.output],
            warnings: plan.diagnostics,
        })
    }
}

struct BuildFont;

impl Operation for BuildFont {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "build-font",
            "重建 NP2 字库",
            vec![
                Field::new("plan", "字体计划 JSON", FieldKind::Path).required(),
                Field::new("source", "原始 font.tmp", FieldKind::Path).required(),
                Field::new("output", "新建字库输出目录", FieldKind::Path).required(),
                Field::new(
                    "face",
                    "Windows 字体名称（可用 | 分隔回退字体）",
                    FieldKind::Text,
                )
                .default(Value::Text("新宋体".into())),
            ],
        );
        spec.description =
            "按字体计划调用 vn-font 重绘 NP2 16×16 字槽，并输出 font.tmp 与清单。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err(Error("字库构建只创建新目录，请使用不存在的输出目录".into()));
        }
        let plan = parameters.path("plan")?.to_path_buf();
        let source = parameters.path("source")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        if !plan.is_file() || !source.is_file() {
            return Err(Error("字体计划和源 font.tmp 都必须是文件".into()));
        }
        validate_new_output(&output)?;
        let face = parameters.text("face")?.to_string();
        Ok(Box::new(PreparedBuildFont {
            plan,
            source,
            output,
            face,
            preview: Preview {
                inputs: vec![
                    parameters.path("plan")?.to_path_buf(),
                    parameters.path("source")?.to_path_buf(),
                ],
                outputs: vec![parameters.path("output")?.to_path_buf()],
                steps: vec![
                    "校验字体计划 schema 与保留 CP932 字槽".into(),
                    "使用同一映射计划重绘 font.tmp".into(),
                    "写入字库哈希和 CPMAIN 扩展码位清单".into(),
                ],
                details: Vec::new(),
            },
        }))
    }
}

struct PreparedBuildFont {
    plan: std::path::PathBuf,
    source: std::path::PathBuf,
    output: std::path::PathBuf,
    face: String,
    preview: Preview,
}

impl PreparedOperation for PreparedBuildFont {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在重建 NP2 字库")?;
        let json = std::fs::read_to_string(&self.plan).map_err(|e| Error(e.to_string()))?;
        let plan: FontPlan = serde_json::from_str(&json).map_err(|e| Error(e.to_string()))?;
        let report: FontArtifactReport =
            font_plan::build_font_artifact(&self.source, &plan, &self.output, &self.face)
                .map_err(Error)?;
        Ok(RunReport {
            summary: format!("已重建 NP2 字库（{} 个字槽）", report.patched_glyphs),
            totals: vec![("重绘字槽".into(), report.patched_glyphs as u64)],
            outputs: vec![report.output_directory],
            warnings: Vec::new(),
        })
    }
}

struct RebuildD88;

impl Operation for RebuildD88 {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "rebuild-d88",
            "重建单张 D88",
            vec![
                Field::new("source", "原始 D88 软碟", FieldKind::Path).required(),
                Field::new("original", "原始解包盘目录", FieldKind::Path).required(),
                Field::new("modified", "译后解包盘目录", FieldKind::Path).required(),
                Field::new("output", "新建 D88 输出文件", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "校验原始 fivec 清单后，把译后工作区按 FAT12 簇链写回固定几何 D88。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err(Error("D88 重建只创建新文件，请使用不存在的输出路径".into()));
        }
        let source = parameters.path("source")?.to_path_buf();
        let original = parameters.path("original")?.to_path_buf();
        let modified = parameters.path("modified")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        if !source.is_file() || !original.is_dir() || !modified.is_dir() {
            return Err(Error(
                "source 必须是文件，original/modified 必须是目录".into(),
            ));
        }
        validate_new_output(&output)?;
        Ok(Box::new(PreparedRebuildD88 {
            source,
            original,
            modified,
            output,
            preview: Preview {
                inputs: vec![
                    parameters.path("source")?.to_path_buf(),
                    parameters.path("original")?.to_path_buf(),
                    parameters.path("modified")?.to_path_buf(),
                ],
                outputs: vec![parameters.path("output")?.to_path_buf()],
                steps: vec![
                    "校验原始 D88、FAT12、簇链和 fivec 文件哈希".into(),
                    "比较译后工作区并分配/释放 FAT12 簇".into(),
                    "写入新 D88 后重新解码校验文件内容".into(),
                ],
                details: Vec::new(),
            },
        }))
    }
}

struct PreparedRebuildD88 {
    source: std::path::PathBuf,
    original: std::path::PathBuf,
    modified: std::path::PathBuf,
    output: std::path::PathBuf,
    preview: Preview,
}

struct RebuildDisks;

impl Operation for RebuildDisks {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "rebuild-disks",
            "批量重建 D88",
            vec![
                Field::new("source", "原始 D88 软碟（按盘序重复）", FieldKind::Paths).required(),
                Field::new("original-root", "原始多盘解包工作区", FieldKind::Path).required(),
                Field::new("modified-root", "译后多盘解包工作区", FieldKind::Path).required(),
                Field::new("output", "新建批量 D88 输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "按 workspace.json 盘序校验全部输入，逐盘重建并在所有盘成功后原子提交输出目录。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        if parameters.flag("overwrite") {
            return Err(Error(
                "批量 D88 重建只创建新目录，请使用不存在的输出目录".into(),
            ));
        }
        let sources = parameters.paths("source")?.to_vec();
        let original_root = parameters.path("original-root")?.to_path_buf();
        let modified_root = parameters.path("modified-root")?.to_path_buf();
        let output = parameters.path("output")?.to_path_buf();
        if sources.is_empty() {
            return Err(Error("至少需要一张 D88 输入镜像".into()));
        }
        if sources.iter().any(|path| !path.is_file()) {
            return Err(Error("source 中每个路径都必须是 D88 文件".into()));
        }
        if !original_root.is_dir() || !modified_root.is_dir() {
            return Err(Error("original-root/modified-root 必须是目录".into()));
        }
        validate_new_output(&output)?;
        Ok(Box::new(PreparedRebuildDisks {
            sources: sources.clone(),
            original_root: original_root.clone(),
            modified_root: modified_root.clone(),
            output: output.clone(),
            preview: Preview {
                inputs: {
                    let mut inputs = sources;
                    inputs.push(original_root);
                    inputs.push(modified_root);
                    inputs
                },
                outputs: vec![output],
                steps: vec![
                    "校验两套 workspace.json、盘序、来源名称和 SHA-256".into(),
                    "逐盘校验 FAT12、文件哈希和译后工作区并重建 D88".into(),
                    "全部成功后原子提交批量输出目录和清单".into(),
                ],
                details: Vec::new(),
            },
        }))
    }
}

struct PreparedRebuildDisks {
    sources: Vec<std::path::PathBuf>,
    original_root: std::path::PathBuf,
    modified_root: std::path::PathBuf,
    output: std::path::PathBuf,
    preview: Preview,
}

impl PreparedOperation for PreparedRebuildDisks {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在批量重建 D88")?;
        let report = rebuild::rebuild_d88_batch_from_workspaces(
            &self.sources,
            &self.original_root,
            &self.modified_root,
            &self.output,
        )
        .map_err(Error)?;
        Ok(RunReport {
            summary: format!(
                "已批量重建 {} 张 D88，改动 {} 个文件",
                report.disks, report.changed_files
            ),
            totals: vec![
                ("软碟".into(), report.disks as u64),
                ("文件".into(), report.files as u64),
                ("改动文件".into(), report.changed_files as u64),
                ("字节".into(), report.bytes as u64),
            ],
            outputs: vec![report.output_directory],
            warnings: Vec::new(),
        })
    }
}

impl PreparedOperation for PreparedRebuildD88 {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在重建 D88")?;
        let report = rebuild::rebuild_d88_from_workspaces(
            &self.source,
            &self.original,
            &self.modified,
            &self.output,
        )
        .map_err(Error)?;
        Ok(RunReport {
            summary: format!("已重建 D88，改动 {} 个文件", report.changed_files),
            totals: vec![
                ("文件".into(), report.files as u64),
                ("改动文件".into(), report.changed_files as u64),
                ("新增簇".into(), report.allocated_clusters as u64),
                ("释放簇".into(), report.released_clusters as u64),
            ],
            outputs: vec![self.output],
            warnings: Vec::new(),
        })
    }
}

fn validate_directory(path: &std::path::Path) -> Result<()> {
    let metadata =
        std::fs::metadata(path).map_err(|e| Error(format!("无法读取 {}: {e}", path.display())))?;
    if !metadata.is_dir() {
        return Err(Error(format!("不是目录: {}", path.display())));
    }
    Ok(())
}

fn validate_new_output(path: &std::path::Path) -> Result<()> {
    if path.exists() {
        return Err(Error(format!("输出已存在: {}", path.display())));
    }
    Ok(())
}

impl PreparedOperation for PreparedJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在解包 PC-98 D88 软碟")?;
        let report = self.prepared.execute().map_err(Error)?;
        Ok(RunReport {
            summary: format!("已解包 {} 张软碟", report.disks),
            totals: vec![
                ("软碟".into(), report.disks as u64),
                ("文件".into(), report.files as u64),
                ("字节".into(), report.bytes as u64),
            ],
            outputs: vec![report.output],
            warnings: report.warnings,
        })
    }
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "银白色的乐园 PC-98",
            env!("CARGO_PKG_VERSION"),
            vec![
                Box::new(Unpack),
                Box::new(ExtractText),
                Box::new(InjectText),
                Box::new(FontPlanOp),
                Box::new(BuildFont),
                Box::new(RebuildD88),
                Box::new(RebuildDisks),
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
