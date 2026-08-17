use aitsuno_freecg98_tool::{
    render_embedded, verify_font_bmp, write_output, RenderReport, ToolError, ToolResult,
};
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> ToolResult<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return interactive(None);
    }
    let first = args[0].to_string_lossy();
    match first.as_ref() {
        "-h" | "--help" | "help" => {
            print_help();
            Ok(())
        }
        "render" => run_render_command(&args[1..]),
        "verify" => run_verify_command(&args[1..]),
        _ if args.len() == 1 => interactive(Some(PathBuf::from(&args[0]))),
        _ => Err(ToolError(
            "unrecognized arguments; use --help for the command contract".to_string(),
        )),
    }
}

fn run_render_command(args: &[OsString]) -> ToolResult<()> {
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0;
    while index < args.len() {
        let argument = args[index].to_string_lossy();
        match argument.as_ref() {
            "-o" | "--output" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| ToolError(format!("{argument} requires a path")))?;
                output = Some(PathBuf::from(value));
            }
            "--overwrite" => overwrite = true,
            "--non-interactive" => {}
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ => {
                return Err(ToolError(format!(
                    "unknown render argument {argument:?}; use --help"
                )))
            }
        }
        index += 1;
    }
    let output = output.ok_or_else(|| {
        ToolError("render requires an explicit --output path in non-interactive mode".to_string())
    })?;
    render_to_path(&output, overwrite)
}

fn run_verify_command(args: &[OsString]) -> ToolResult<()> {
    let mut input = None;
    let mut index = 0;
    while index < args.len() {
        let argument = args[index].to_string_lossy();
        match argument.as_ref() {
            "-i" | "--input" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| ToolError(format!("{argument} requires a path")))?;
                input = Some(PathBuf::from(value));
            }
            "--non-interactive" => {}
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            _ if !argument.starts_with('-') && input.is_none() => {
                input = Some(PathBuf::from(&args[index]));
            }
            _ => {
                return Err(ToolError(format!(
                    "unknown verify argument {argument:?}; use --help"
                )))
            }
        }
        index += 1;
    }
    let input = input.ok_or_else(|| {
        ToolError("verify requires an explicit BMP path in non-interactive mode".to_string())
    })?;
    verify_path(&input)
}

fn render_to_path(output: &Path, overwrite: bool) -> ToolResult<()> {
    let (bmp, report) = render_embedded()?;
    write_output(output, &bmp, overwrite)?;
    print_render_report(&report, output);
    Ok(())
}

fn verify_path(input: &Path) -> ToolResult<()> {
    let bytes = fs::read(input)
        .map_err(|error| ToolError(format!("cannot read BMP '{}': {error}", input.display())))?;
    let report = verify_font_bmp(&bytes)?;
    println!("[verify] mapping_entries={}", report.mapping_entries);
    println!("[verify] unique_slots={}", report.unique_slots);
    println!("[verify] nonempty_slots={}", report.nonempty_slots);
    println!("[verify] matching_slots={}", report.matching_slots);
    println!("[verify] bmp_bytes={}", report.bmp_bytes);
    println!("[verify] input={}", input.display());
    if report.nonempty_slots != report.mapping_entries {
        return Err(ToolError(format!(
            "only {}/{} mapped slots are nonempty",
            report.nonempty_slots, report.mapping_entries
        )));
    }
    if report.matching_slots != report.mapping_entries {
        return Err(ToolError(format!(
            "only {}/{} mapped slots match the approved 16px monochrome baseline",
            report.matching_slots, report.mapping_entries
        )));
    }
    Ok(())
}

fn print_render_report(report: &RenderReport, output: &Path) {
    println!("[font] mapping_entries={}", report.mapping_entries);
    println!("[font] unique_slots={}", report.unique_slots);
    println!(
        "[font] source_nonempty_slots={}",
        report.source_nonempty_slots
    );
    println!(
        "[font] rendered_nonempty_slots={}",
        report.rendered_nonempty_slots
    );
    println!("[font] output_bytes={}", report.output_bytes);
    for evidence in &report.focus {
        println!(
            "[font] target={} carrier={} cp932={:02X}{:02X} jis={:04X} tile=({}, {}) source_black_pixels={} rendered_black_pixels={}",
            evidence.target,
            evidence.carrier,
            evidence.shift_jis[0],
            evidence.shift_jis[1],
            evidence.slot.jis_code(),
            evidence.slot.tile_x,
            evidence.slot.tile_y,
            evidence.source_black_pixels,
            evidence.rendered_black_pixels
        );
    }
    println!("[font] output={}", output.display());
}

fn interactive(prefill: Option<PathBuf>) -> ToolResult<()> {
    let mut output_prefill = prefill;
    loop {
        println!();
        println!("FREECG98 字体重绘工具");
        println!("1. 使用内嵌资源生成 FREECG98.BMP");
        println!("2. 验证已有 FREECG98.BMP");
        println!("0. 退出");
        if output_prefill.is_some() {
            if !interactive_render(output_prefill.take())? {
                return Ok(());
            }
            continue;
        }
        let Some(choice) = prompt("选择")? else {
            return Ok(());
        };
        match choice.trim() {
            "0" => return Ok(()),
            "1" => {
                interactive_render(None)?;
            }
            "2" => {
                interactive_verify()?;
            }
            _ => println!("无效选择，请重试。"),
        }
    }
}

fn interactive_render(prefill: Option<PathBuf>) -> ToolResult<bool> {
    let suggested = prefill.unwrap_or_else(|| PathBuf::from("FREECG98.CN.BMP"));
    let Some(value) = prompt(&format!("输出路径 [{}]", suggested.display()))? else {
        return Ok(false);
    };
    let output = if value.trim().is_empty() {
        suggested
    } else {
        PathBuf::from(strip_drag_quotes(value.trim()))
    };
    let mut overwrite = false;
    if output.exists() {
        println!("目标已存在：{}", output.display());
        let Some(choice) = prompt("输入 O 覆盖、E 修改路径、C 取消")? else {
            return Ok(false);
        };
        match choice.trim().to_ascii_lowercase().as_str() {
            "o" => overwrite = true,
            "e" => return interactive_render(None),
            _ => {
                println!("已取消。");
                return Ok(true);
            }
        }
    }
    println!("资源：内嵌基础 BMP、3025 条映射、内嵌 16px 单色字形表");
    println!("输出：{}", output.display());
    println!("覆盖：{}", if overwrite { "是" } else { "否" });
    let Some(confirm) = prompt("确认生成？[y/N]")? else {
        return Ok(false);
    };
    if !confirm.trim().eq_ignore_ascii_case("y") {
        println!("已取消，未写入文件。");
        return Ok(true);
    }
    match render_to_path(&output, overwrite) {
        Ok(()) => println!("生成完成。"),
        Err(error) => println!("操作失败：{error}"),
    }
    Ok(true)
}

fn interactive_verify() -> ToolResult<()> {
    let Some(value) = prompt("待验证 BMP 路径（0 返回）")? else {
        return Ok(());
    };
    if value.trim() == "0" || value.trim().is_empty() {
        return Ok(());
    }
    let input = PathBuf::from(strip_drag_quotes(value.trim()));
    if let Err(error) = verify_path(&input) {
        println!("验证失败：{error}");
    }
    Ok(())
}

fn prompt(label: &str) -> ToolResult<Option<String>> {
    print!("{label}: ");
    io::stdout()
        .flush()
        .map_err(|error| ToolError(format!("cannot flush console: {error}")))?;
    let mut value = String::new();
    let count = io::stdin()
        .read_line(&mut value)
        .map_err(|error| ToolError(format!("cannot read console input: {error}")))?;
    if count == 0 {
        return Ok(None);
    }
    Ok(Some(value.trim_end_matches(['\r', '\n']).to_string()))
}

fn strip_drag_quotes(value: &str) -> &str {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn print_help() {
    println!("aitsuno_freecg98_tool");
    println!();
    println!("内嵌基础 FREECG98.BMP、CN->JP 载体映射和 16px 单色字形表，生成时只需指定输出。");
    println!();
    println!("非交互：");
    println!("  aitsuno_freecg98_tool.exe render --output <OUTPUT.BMP> [--overwrite] [--non-interactive]");
    println!("  aitsuno_freecg98_tool.exe verify --input <OUTPUT.BMP> [--non-interactive]");
    println!();
    println!("交互/拖放：");
    println!("  aitsuno_freecg98_tool.exe");
    println!("  aitsuno_freecg98_tool.exe <OUTPUT.BMP>");
    println!();
    println!("无参数或仅一个路径参数会进入持续交互会话；确认前不会写文件。");
    println!("默认拒绝已有输出，只有 --overwrite 或交互确认才覆盖。");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_quotes_are_removed_only_as_a_pair() {
        assert_eq!(
            strip_drag_quotes("\"C:\\A & B\\font.bmp\""),
            "C:\\A & B\\font.bmp"
        );
        assert_eq!(strip_drag_quotes("\"broken"), "\"broken");
    }
}
