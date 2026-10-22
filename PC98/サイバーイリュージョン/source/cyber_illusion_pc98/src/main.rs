use cyber_illusion_pc98::{hdi_pack, workflow};
use std::{path::PathBuf, process::ExitCode};
use vn_cli::{
    Error, Field, FieldKind, Operation, OperationSpec, Panel, Parameters, PreparedOperation,
    Result, Value,
};

struct Export;

impl Operation for Export {
    fn spec(&self) -> OperationSpec {
        let fields = vec![
            Field::new("input-dir", "已安装的游戏目录", FieldKind::Path).required(),
            Field::new("output", "新建文本工作区目录", FieldKind::Path).required(),
        ];
        let mut spec = OperationSpec::new("export", "提取脚本与翻译 JSON", fields);
        spec.writes = true;
        spec.description =
            "读取完整 DOS 游戏目录，通过 AE 的 eLZ0 规则展开场景，生成来源绑定的主提取与 vn-text JSON。"
                .into();
        spec
    }

    fn prefill(&self, paths: &[PathBuf], parameters: &mut Parameters) -> Result<()> {
        if let Some(path) = paths.first() {
            let directory = if path.is_dir() {
                path.clone()
            } else {
                path.parent()
                    .ok_or_else(|| Error("拖入文件没有父目录".into()))?
                    .to_path_buf()
            };
            parameters.set("input-dir", Value::Path(directory));
        }
        Ok(())
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        reject_overwrite(parameters)?;
        let job =
            workflow::prepare_export(parameters.path("input-dir")?, parameters.path("output")?)
                .map_err(Error)?;
        Ok(Box::new(job))
    }
}

struct Import;

impl Operation for Import {
    fn spec(&self) -> OperationSpec {
        let fields = vec![
            Field::new("input-dir", "当前原始游戏目录", FieldKind::Path).required(),
            Field::new("workspace", "原始文本工作区", FieldKind::Path).required(),
            Field::new(
                "translation",
                "翻译 JSON 文件或目录（可多选）",
                FieldKind::Paths,
            )
            .required(),
            Field::new("output", "新建回注游戏目录", FieldKind::Path).required(),
        ];
        let mut spec = OperationSpec::new("import", "回注译文并重建场景文件", fields);
        spec.writes = true;
        spec.description = "重新解析当前来源并核对工作区绑定，按 NP2 字库映射编码译文、处理变长记录并生成独立的完整 DOS 游戏目录。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        reject_overwrite(parameters)?;
        let translations = parameters.paths("translation")?.to_vec();
        let job = workflow::prepare_import(
            parameters.path("input-dir")?,
            parameters.path("workspace")?,
            &translations,
            parameters.path("output")?,
        )
        .map_err(Error)?;
        Ok(Box::new(job))
    }
}

struct PackHdi;

impl Operation for PackHdi {
    fn spec(&self) -> OperationSpec {
        let fields = vec![
            Field::new("blank-hdi", "空白 PC-98 HDI", FieldKind::Path).required(),
            Field::new("dos-root", "PC-98 DOS 解包目录", FieldKind::Path).required(),
            Field::new("game-root", "Cyber 回注游戏目录", FieldKind::Path).required(),
            Field::new("output", "新建硬盘包目录", FieldKind::Path).required(),
        ];
        let mut spec = OperationSpec::new("pack-hdi", "生成可启动 Cyber HDI", fields);
        spec.writes = true;
        spec.description = "保留空白 HDI 几何，复用 PC-98 IPL1 与分区前的 DOS 装载区，建立 FAT12 分区并加入 DOS 5 和回注后的 Cyber 游戏文件；NP2 字库作为外部配套文件输出。".into();
        spec
    }

    fn prepare(&self, parameters: &Parameters) -> Result<Box<dyn PreparedOperation>> {
        reject_overwrite(parameters)?;
        let job = hdi_pack::prepare(
            parameters.path("blank-hdi")?,
            parameters.path("dos-root")?,
            parameters.path("game-root")?,
            parameters.path("output")?,
        )
        .map_err(Error)?;
        Ok(Box::new(job))
    }
}

fn reject_overwrite(parameters: &Parameters) -> Result<()> {
    if parameters.flag("overwrite") {
        Err("本工具只创建新目录，不支持覆盖；请指定一个尚不存在的输出路径".into())
    } else {
        Ok(())
    }
}

fn run() -> Result<()> {
    Panel::new(
        std::env::current_exe()?.into_os_string(),
        "サイバーイリュージョン PC-98 文本工具",
        env!("CARGO_PKG_VERSION"),
        vec![Box::new(Export), Box::new(Import), Box::new(PackHdi)],
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
