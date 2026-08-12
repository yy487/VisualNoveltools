use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use farthest2015_cd_tool::encoding::TextEncoding;
use farthest2015_cd_tool::ops::{
    extract_directory, inject_directory, verify_directory, ExtractReport, InjectReport, ScanReport,
};
use farthest2015_cd_tool::ToolResult;

#[derive(Debug, Parser)]
#[command(
    name = "farthest2015-cd-tool",
    version,
    about = "Structure-aware JSON extraction/injection for Farthest2015 .cd scripts",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(value_name = "PATH", num_args = 0..=3)]
    paths: Vec<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a file or directory: MD5, CRC16, sizes, and text structures.
    Verify {
        #[arg(long)]
        source: PathBuf,
        #[arg(long, default_value = "cp932")]
        text_encoding: String,
    },
    /// Extract numbered scenario text to UTF-8 JSON.
    Extract {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value = "cp932")]
        text_encoding: String,
        #[arg(long)]
        overwrite: bool,
    },
    /// Inject UTF-8 JSON translations and copy a complete patched tree.
    Inject {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        translation: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value = "cp932")]
        source_encoding: String,
        #[arg(long, default_value = "cp932")]
        target_encoding: String,
        #[arg(long)]
        overwrite: bool,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ERROR: {error}");
        std::process::exit(1);
    }
}

fn run() -> ToolResult<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Verify {
            source,
            text_encoding,
        }) => {
            let encoding = TextEncoding::parse(&text_encoding)?;
            let report = verify_directory(&source, encoding)?;
            print_scan(&report);
        }
        Some(Command::Extract {
            source,
            output,
            text_encoding,
            overwrite,
        }) => {
            let encoding = TextEncoding::parse(&text_encoding)?;
            let report = extract_directory(&source, &output, encoding, overwrite)?;
            print_extract(&report);
        }
        Some(Command::Inject {
            source,
            translation,
            output,
            source_encoding,
            target_encoding,
            overwrite,
        }) => {
            let source_encoding = TextEncoding::parse(&source_encoding)?;
            let target_encoding = TextEncoding::parse(&target_encoding)?;
            let report = inject_directory(
                &source,
                &translation,
                &output,
                source_encoding,
                target_encoding,
                overwrite,
            )?;
            print_inject(&report);
        }
        None => interactive(cli.paths)?,
    }
    Ok(())
}

fn interactive(prefill: Vec<PathBuf>) -> ToolResult<()> {
    println!("Farthest2015 CD 文本工具（交互模式）");
    if !prefill.is_empty() {
        println!("已载入拖放/路径参数；所有路径仍可编辑，确认前不会写入。 ");
    }
    loop {
        println!();
        println!("1. 校验目录");
        println!("2. 提取 UTF-8 JSON");
        println!("3. 注入并生成完整输出目录");
        println!("0. 退出");
        let Some(choice) = prompt("请选择", None)? else {
            return Ok(());
        };
        let result = match choice.trim() {
            "1" => interactive_verify(&prefill),
            "2" => interactive_extract(&prefill),
            "3" => interactive_inject(&prefill),
            "0" => return Ok(()),
            _ => {
                println!("请输入 0、1、2 或 3。");
                continue;
            }
        };
        if let Err(error) = result {
            eprintln!("操作未完成：{error}");
        }
    }
}

fn interactive_verify(prefill: &[PathBuf]) -> ToolResult<()> {
    let source = prompt_path("源目录", prefill.first())?;
    let encoding = prompt_encoding("文本编码", TextEncoding::Cp932)?;
    println!("将只读校验：{}（{}）", source.display(), encoding.label());
    if !confirm("开始校验")? {
        println!("已取消。");
        return Ok(());
    }
    let report = verify_directory(&source, encoding)?;
    print_scan(&report);
    Ok(())
}

fn interactive_extract(prefill: &[PathBuf]) -> ToolResult<()> {
    let source = prompt_path("源目录", prefill.first())?;
    let output = prompt_path("JSON 输出目录", prefill.get(1))?;
    let encoding = prompt_encoding("源文本编码", TextEncoding::Cp932)?;
    let overwrite = if output.exists() {
        confirm("输出已存在，允许完整替换")?
    } else {
        false
    };
    println!("源目录：{}", source.display());
    println!("输出目录：{}", output.display());
    println!("源编码：{}；覆盖：{}", encoding.label(), yes_no(overwrite));
    if !confirm("确认提取")? {
        println!("已取消，未写入。");
        return Ok(());
    }
    let report = extract_directory(&source, &output, encoding, overwrite)?;
    print_extract(&report);
    Ok(())
}

fn interactive_inject(prefill: &[PathBuf]) -> ToolResult<()> {
    let source = prompt_path("源目录", prefill.first())?;
    let translation = prompt_path("翻译 JSON 目录", prefill.get(1))?;
    let output = prompt_path("补丁输出目录", prefill.get(2))?;
    let source_encoding = prompt_encoding("源文本编码", TextEncoding::Cp932)?;
    let target_encoding = prompt_encoding("写回文本编码", TextEncoding::Cp932)?;
    let overwrite = if output.exists() {
        confirm("输出已存在，允许完整替换")?
    } else {
        false
    };
    println!("源目录：{}", source.display());
    println!("翻译目录：{}", translation.display());
    println!("输出目录：{}", output.display());
    println!(
        "编码：{} -> {}；覆盖：{}",
        source_encoding.label(),
        target_encoding.label(),
        yes_no(overwrite)
    );
    if !confirm("确认注入")? {
        println!("已取消，未写入。");
        return Ok(());
    }
    let report = inject_directory(
        &source,
        &translation,
        &output,
        source_encoding,
        target_encoding,
        overwrite,
    )?;
    print_inject(&report);
    Ok(())
}

fn prompt_path(label: &str, default: Option<&PathBuf>) -> ToolResult<PathBuf> {
    let default_text = default.map(|path| path.to_string_lossy().into_owned());
    loop {
        let Some(value) = prompt(label, default_text.as_deref())? else {
            return Err("输入已结束".to_string());
        };
        let value = strip_optional_quotes(value.trim());
        if !value.is_empty() {
            return Ok(PathBuf::from(value));
        }
        println!("路径不能为空。");
    }
}

fn prompt_encoding(label: &str, default: TextEncoding) -> ToolResult<TextEncoding> {
    loop {
        let Some(value) = prompt(label, Some(default.label()))? else {
            return Err("输入已结束".to_string());
        };
        match TextEncoding::parse(value.trim()) {
            Ok(encoding) => return Ok(encoding),
            Err(error) => println!("{error}"),
        }
    }
}

fn confirm(label: &str) -> ToolResult<bool> {
    let Some(value) = prompt(&format!("{label}？(y/N)"), Some("N"))? else {
        return Ok(false);
    };
    Ok(matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "y" | "yes" | "是"
    ))
}

fn prompt(label: &str, default: Option<&str>) -> ToolResult<Option<String>> {
    match default {
        Some(default) => print!("{label} [{default}]: "),
        None => print!("{label}: "),
    }
    io::stdout()
        .flush()
        .map_err(|error| format!("cannot flush console: {error}"))?;
    let mut input = String::new();
    let read = io::stdin()
        .read_line(&mut input)
        .map_err(|error| format!("cannot read console input: {error}"))?;
    if read == 0 {
        return Ok(None);
    }
    let trimmed = input.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() {
        Ok(Some(default.unwrap_or_default().to_string()))
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn strip_optional_quotes(value: &str) -> &str {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "是"
    } else {
        "否"
    }
}

fn print_scan(report: &ScanReport) {
    println!("校验完成：");
    println!(
        "  文件 {}；场景 {}；标签库 {}（{} 块 / {} 标签）；变量库 {}（{} 变量）；未知 {}",
        report.files,
        report.scenarios,
        report.label_databases,
        report.label_blocks,
        report.labels,
        report.variable_databases,
        report.variables,
        report.unknown_files
    );
    println!(
        "  正文 {}；选项 {}；超链接 {}（跨连续显示组 {}）；外部注音 {}；字体控制 {}",
        report.text_entries,
        report.choice_entries,
        report.hyperlinks,
        report.continuation_hyperlinks,
        report.ruby_controls,
        report.font_controls
    );
}

fn print_extract(report: &ExtractReport) {
    println!(
        "提取完成：{} 个 JSON，{} 条可翻译记录。",
        report.json_files, report.json_entries
    );
    print_scan(&report.scan);
}

fn print_inject(report: &InjectReport) {
    println!(
        "注入完成：{} 个 JSON / {} 条记录；改写 {}，原样 {}；复制 {} 个文件。",
        report.json_files,
        report.json_entries,
        report.patched_entries,
        report.unchanged_entries,
        report.copied_files
    );
    println!(
        "  清理旧物理断行全角空格 {} 个；无法唯一映射而保留 {} 个。",
        report.removed_join_spaces, report.ambiguous_join_spaces
    );
    for warning in &report.warnings {
        println!("WARNING: {warning}");
    }
}
