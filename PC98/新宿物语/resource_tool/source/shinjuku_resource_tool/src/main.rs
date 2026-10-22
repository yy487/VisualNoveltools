use shinjuku_resource_tool::{
    prepare_batch, validate_output_target, write_prepared, PreparedBatch,
};
use std::path::PathBuf;
use std::process::ExitCode;
use vn_cli::{
    Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation, Preview,
    Progress, Result, RunReport, Value,
};

struct DecryptOvl;

impl Operation for DecryptOvl {
    fn spec(&self) -> OperationSpec {
        let mut spec = OperationSpec::new(
            "decrypt-ovl",
            "去除 FAIRYTALE OVL 自解密壳",
            vec![
                Field::new("input", "一个或多个 OVL 文件", FieldKind::Paths).required(),
                Field::new("output", "独立输出目录", FieldKind::Path).required(),
            ],
        );
        spec.description =
            "识别 TC0 壳并按运行时算法解密 MZ 负载，生成可直接载入 IDA 的独立副本。".into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if paths.is_empty() {
            return Ok(());
        }
        parameters.set("input", Value::Paths(paths.to_vec()));
        if let Some(first) = paths.first() {
            let parent = first.parent().unwrap_or_else(|| std::path::Path::new("."));
            parameters.set("output", Value::Path(parent.join("decrypted")));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        let output = std::path::absolute(parameters.path("output")?)?;
        let overwrite = parameters.flag("overwrite");
        validate_output_target(&output, overwrite).map_err(vn_cli::Error)?;
        let prepared = prepare_batch(parameters.paths("input")?).map_err(vn_cli::Error)?;
        let details = prepared
            .files
            .iter()
            .map(|item| {
                let decrypted_bytes = item
                    .info
                    .layers
                    .iter()
                    .map(|layer| layer.encrypted_size as u64)
                    .sum::<u64>();
                format!(
                    "{}: 去除 {} 层，累计解密 {} 字节；入口 {:04X}:{:04X}；SHA-256 {} -> {}",
                    item.source.display(),
                    item.info.layers.len(),
                    decrypted_bytes,
                    item.info.final_cs,
                    item.info.final_ip,
                    item.source_sha256,
                    item.output_sha256
                )
            })
            .collect();
        Ok(Box::new(DecryptJob {
            preview: Preview {
                inputs: prepared
                    .files
                    .iter()
                    .map(|item| item.source.clone())
                    .collect(),
                outputs: vec![output.clone()],
                steps: vec![
                    "验证 MZ、TC0 入口、解密 stub 与参数边界".into(),
                    "按运行时 LCG/XOR 解密负载并重建 0x20 字节 MZ 头".into(),
                    "事务式写出 IDA 输入副本".into(),
                ],
                details,
            },
            prepared,
            output,
            overwrite,
        }))
    }
}

struct DecryptJob {
    preview: Preview,
    prepared: PreparedBatch,
    output: PathBuf,
    overwrite: bool,
}

impl PreparedOperation for DecryptJob {
    fn preview(&self) -> &Preview {
        &self.preview
    }

    fn execute(self: Box<Self>, progress: &mut dyn Progress) -> Result<RunReport> {
        progress.report("正在生成去壳 IDA 输入")?;
        let report =
            write_prepared(self.prepared, &self.output, self.overwrite).map_err(vn_cli::Error)?;
        Ok(RunReport {
            summary: "OVL 自解密壳处理完成；原文件未修改。".into(),
            totals: vec![
                ("文件数".into(), report.files as u64),
                ("输入字节".into(), report.input_bytes),
                ("输出字节".into(), report.output_bytes),
            ],
            outputs: vec![report.output_root],
            warnings: Vec::new(),
        })
    }
}

fn main() -> ExitCode {
    let result = (|| {
        Panel::new(
            std::env::current_exe()?.into_os_string(),
            "《新宿物語》资源研究工具",
            env!("CARGO_PKG_VERSION"),
            vec![Box::new(DecryptOvl)],
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
