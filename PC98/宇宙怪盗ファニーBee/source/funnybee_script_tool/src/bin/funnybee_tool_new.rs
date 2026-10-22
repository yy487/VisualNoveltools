use funnybee_script_tool::{
    export_plain_text, extract_translation_with_gaiji, import_plain_text,
    inject_translation_localized, prepare_font, recursive_extract_txt, recursive_pack_txt,
    GaijiTable, GlyphTable, InjectOptions, RecursivePackOptions, SubstitutionMap,
};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"funnybee_tool_new 0.3.2

《宇宙怪盗ファニーBee》无前缀 UTF-8 TXT 一体化提取/注入/回封工具。

用法:
  funnybee_tool_new.exe extract-recursive --input <FILE_OR_DIR> --output <WORKSPACE> [--adv <ADV.COM>] [--overwrite]
  funnybee_tool_new.exe pack-recursive --source <FILE_OR_DIR> --workspace <WORKSPACE> --output <FILE_OR_DIR> [--adv <ADV.COM>] [--max-columns N] [--overwrite]
  funnybee_tool_new.exe extract --input <MEMBER.BIN> --adv <ADV.COM> --output <FILE.TXT> [--overwrite]
  funnybee_tool_new.exe inject --input <MEMBER.BIN> --translation <FILE.TXT> --adv <ADV.COM> --output <MEMBER.BIN> [--max-columns N] [--overwrite]

每条消息占一个物理行，没有 txt= 前缀。游戏内 0x52 换行写成字面量 \n，原文反斜杠
写成 \\；行首尾空格和 H/X 控制原样保留。GAIJI、中文替换表和基准 font.tmp 已内置。
回封中文时会自动生成匹配的 font.tmp；不会覆盖源脚本、源封包或源字库。
"#;

#[derive(Clone, Copy, Debug)]
enum Mode {
    ExtractRecursive,
    PackRecursive,
    ExtractSingle,
    InjectSingle,
}

#[derive(Debug)]
struct Invocation {
    mode: Mode,
    input: PathBuf,
    translation: Option<PathBuf>,
    workspace: Option<PathBuf>,
    adv: Option<PathBuf>,
    output: PathBuf,
    max_columns: Option<usize>,
    overwrite: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("错误: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return interactive_loop(None);
    }
    if args.len() == 1 {
        if matches!(args[0].to_str(), Some("-h" | "--help")) {
            print!("{HELP}");
            return Ok(());
        }
        return interactive_loop(Some(PathBuf::from(&args[0])));
    }
    execute(&parse_args(args)?)
}

fn parse_args(args: Vec<OsString>) -> Result<Invocation, String> {
    let mode = match args.first().and_then(|value| value.to_str()) {
        Some("extract-recursive") => Mode::ExtractRecursive,
        Some("pack-recursive") => Mode::PackRecursive,
        Some("extract") => Mode::ExtractSingle,
        Some("inject") => Mode::InjectSingle,
        _ => return Err("未知命令；使用 --help 查看用法".to_string()),
    };
    let mut input = None;
    let mut translation = None;
    let mut workspace = None;
    let mut adv = None;
    let mut output = None;
    let mut max_columns = None;
    let mut overwrite = false;
    let mut index = 1usize;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| "命令选项必须为 Unicode".to_string())?;
        index += 1;
        if option == "--overwrite" {
            if overwrite {
                return Err("--overwrite 只能指定一次".to_string());
            }
            overwrite = true;
            continue;
        }
        let value = args.get(index).ok_or_else(|| format!("{option} 缺少值"))?;
        index += 1;
        match option {
            "--input" | "--source" | "-i" => set_path(&mut input, value, option)?,
            "--translation" | "-t" => set_path(&mut translation, value, option)?,
            "--workspace" | "-w" => set_path(&mut workspace, value, option)?,
            "--adv" => set_path(&mut adv, value, option)?,
            "--output" | "-o" => set_path(&mut output, value, option)?,
            "--max-columns" => {
                if max_columns.is_some() {
                    return Err("--max-columns 只能指定一次".to_string());
                }
                let text = value
                    .to_str()
                    .ok_or_else(|| "--max-columns 不是 Unicode 数字".to_string())?;
                let parsed = text
                    .parse::<usize>()
                    .map_err(|_| format!("无效 --max-columns: {text}"))?;
                if parsed == 0 {
                    return Err("--max-columns 必须大于 0".to_string());
                }
                max_columns = Some(parsed);
            }
            _ => return Err(format!("未知参数: {option}")),
        }
    }
    let needs_translation = matches!(mode, Mode::InjectSingle);
    let needs_workspace = matches!(mode, Mode::PackRecursive);
    let needs_adv = matches!(mode, Mode::ExtractSingle | Mode::InjectSingle);
    let accepts_max_columns = matches!(mode, Mode::PackRecursive | Mode::InjectSingle);
    if needs_adv && adv.is_none() {
        return Err("散脚本 extract/inject 需要 --adv <ADV.COM>".to_string());
    }
    if !needs_translation && translation.is_some() {
        return Err("只有 inject 接受 --translation".to_string());
    }
    if !needs_workspace && workspace.is_some() {
        return Err("只有 pack-recursive 接受 --workspace".to_string());
    }
    if !accepts_max_columns && max_columns.is_some() {
        return Err("只有 inject/pack-recursive 接受 --max-columns".to_string());
    }
    Ok(Invocation {
        mode,
        input: input.ok_or_else(|| "缺少 --input".to_string())?,
        translation: if needs_translation {
            Some(translation.ok_or_else(|| "缺少 --translation".to_string())?)
        } else {
            None
        },
        workspace: if needs_workspace {
            Some(workspace.ok_or_else(|| "缺少 --workspace".to_string())?)
        } else {
            None
        },
        adv,
        output: output.ok_or_else(|| "缺少 --output".to_string())?,
        max_columns,
        overwrite,
    })
}

fn set_path(
    destination: &mut Option<PathBuf>,
    value: &OsString,
    option: &str,
) -> Result<(), String> {
    if destination.replace(PathBuf::from(value)).is_some() {
        return Err(format!("{option} 只能指定一次"));
    }
    Ok(())
}

fn execute(invocation: &Invocation) -> Result<(), String> {
    match invocation.mode {
        Mode::ExtractRecursive => {
            let report = recursive_extract_txt(
                &invocation.input,
                invocation.adv.as_deref(),
                &invocation.output,
                invocation.overwrite,
            )?;
            println!(
                "完成: roots={} fdi={} dat={} scripts={} entries={} output={}",
                report.scanned_root_files,
                report.fdi_images,
                report.dat_archives,
                report.scripts,
                report.entries,
                report.output.display()
            );
            for dat in report.dat_outputs {
                println!("DAT: {dat}");
            }
            for warning in report.warnings {
                eprintln!("警告: {warning}");
            }
            Ok(())
        }
        Mode::PackRecursive => {
            let report = recursive_pack_txt(
                &invocation.input,
                invocation.workspace.as_deref().expect("pack workspace"),
                invocation.adv.as_deref(),
                &invocation.output,
                RecursivePackOptions {
                    max_columns: invocation.max_columns,
                    overwrite: invocation.overwrite,
                },
            )?;
            println!(
                "完成: containers={} scripts={} changed_scripts={} changed_dat={} changed_fdi={} font_glyphs={} output={}",
                report.containers,
                report.scripts,
                report.changed_scripts,
                report.changed_dat_archives,
                report.changed_fdi_images,
                report.font_glyphs,
                report.output.display()
            );
            if let Some(path) = report.font_output {
                println!("字库: {}", path.display());
            }
            Ok(())
        }
        Mode::ExtractSingle => extract_single(
            &invocation.input,
            invocation.adv.as_deref().expect("single ADV"),
            &invocation.output,
            invocation.overwrite,
        ),
        Mode::InjectSingle => inject_single(
            &invocation.input,
            invocation
                .translation
                .as_deref()
                .expect("single translation"),
            invocation.adv.as_deref().expect("single ADV"),
            &invocation.output,
            invocation.max_columns,
            invocation.overwrite,
        ),
    }
}

fn extract_single(input: &Path, adv: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    if !input.is_file() {
        return Err(format!("脚本输入不是文件: {}", input.display()));
    }
    if !adv.is_file() {
        return Err(format!("ADV 输入不是文件: {}", adv.display()));
    }
    ensure_distinct_file_paths(input, output)?;
    let source =
        fs::read(input).map_err(|error| format!("无法读取 {}: {error}", input.display()))?;
    let adv_bytes =
        fs::read(adv).map_err(|error| format!("无法读取 {}: {error}", adv.display()))?;
    let glyphs = GlyphTable::from_adv(&adv_bytes)?;
    let gaiji = GaijiTable::builtin()?;
    let source_name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("脚本文件名无法表示为 Unicode: {}", input.display()))?;
    let document = extract_translation_with_gaiji(&source, &glyphs, source_name, &gaiji)?;
    let bytes = export_plain_text(&document)?;
    write_file_transactional(output, &bytes, overwrite)?;
    println!(
        "完成: entries={} bytes={} output={}",
        document.entries.len(),
        bytes.len(),
        output.display()
    );
    Ok(())
}

fn inject_single(
    input: &Path,
    translation: &Path,
    adv: &Path,
    output: &Path,
    max_columns: Option<usize>,
    overwrite: bool,
) -> Result<(), String> {
    if !input.is_file() {
        return Err(format!("脚本输入不是文件: {}", input.display()));
    }
    if !translation.is_file() {
        return Err(format!("TXT 翻译不是文件: {}", translation.display()));
    }
    if !adv.is_file() {
        return Err(format!("ADV 输入不是文件: {}", adv.display()));
    }
    ensure_distinct_file_paths(input, output)?;
    let source =
        fs::read(input).map_err(|error| format!("无法读取 {}: {error}", input.display()))?;
    let txt = fs::read(translation)
        .map_err(|error| format!("无法读取 TXT {}: {error}", translation.display()))?;
    let adv_bytes =
        fs::read(adv).map_err(|error| format!("无法读取 {}: {error}", adv.display()))?;
    let glyphs = GlyphTable::from_adv(&adv_bytes)?;
    let gaiji = GaijiTable::builtin()?;
    let substitutions = SubstitutionMap::embedded()?;
    let source_name = input
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("脚本文件名无法表示为 Unicode: {}", input.display()))?;
    let mut document = extract_translation_with_gaiji(&source, &glyphs, source_name, &gaiji)?;
    import_plain_text(&mut document, &txt)
        .map_err(|error| format!("{}: {error}", translation.display()))?;
    let report = inject_translation_localized(
        &source,
        &glyphs,
        source_name,
        &document,
        InjectOptions { max_columns },
        &gaiji,
        &substitutions,
    )?;
    let font = prepare_font(&report.font_requests, &report.literal_characters)?;
    let font_output = sibling_font_path(output)?;
    if font_output.exists() && !overwrite {
        return Err(format!(
            "字库输出已存在，需明确加 --overwrite: {}",
            font_output.display()
        ));
    }
    write_file_transactional(output, &report.bytes, overwrite)?;
    write_file_transactional(&font_output, &font.bytes, overwrite)?;
    println!(
        "完成: entries={} changed={} wraps={} dynamic_wrap_skips={} font_glyphs={} bytes={} output={} font={}",
        report.entries,
        report.changed_entries,
        report.inserted_newlines,
        report.dynamic_wrap_skips,
        font.patched_glyphs,
        report.bytes.len(),
        output.display(),
        font_output.display()
    );
    Ok(())
}

fn sibling_font_path(output: &Path) -> Result<PathBuf, String> {
    let stem = output
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("输出名无法表示为 Unicode: {}", output.display()))?;
    Ok(usable_parent(output).join(format!("{stem}_font.tmp")))
}

fn ensure_distinct_file_paths(input: &Path, output: &Path) -> Result<(), String> {
    let input = fs::canonicalize(input)
        .map_err(|error| format!("无法解析输入路径 {}: {error}", input.display()))?;
    if output.exists() {
        let output = fs::canonicalize(output)
            .map_err(|error| format!("无法解析输出路径 {}: {error}", output.display()))?;
        if input == output {
            return Err("输入和输出不能是同一文件".to_string());
        }
    }
    Ok(())
}

fn write_file_transactional(output: &Path, bytes: &[u8], overwrite: bool) -> Result<(), String> {
    if output.exists() && !overwrite {
        return Err(format!(
            "输出已存在，需明确加 --overwrite: {}",
            output.display()
        ));
    }
    if output.is_dir() {
        return Err(format!("文件输出路径是目录: {}", output.display()));
    }
    let parent = usable_parent(output);
    fs::create_dir_all(parent)
        .map_err(|error| format!("无法建立输出父目录 {}: {error}", parent.display()))?;
    let name = output
        .file_name()
        .ok_or_else(|| format!("输出路径不能是文件系统根: {}", output.display()))?;
    let mut temporary_name = OsString::from(".");
    temporary_name.push(name);
    temporary_name.push(format!(".tmp-{}", std::process::id()));
    let temporary = parent.join(temporary_name);
    if temporary.exists() {
        return Err(format!("事务临时文件已存在: {}", temporary.display()));
    }
    fs::write(&temporary, bytes)
        .map_err(|error| format!("无法写入临时文件 {}: {error}", temporary.display()))?;
    let result = commit_file(&temporary, output, overwrite);
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn commit_file(temporary: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    if !output.exists() {
        return fs::rename(temporary, output)
            .map_err(|error| format!("无法提交输出 {}: {error}", output.display()));
    }
    if !overwrite {
        return Err(format!("输出已存在: {}", output.display()));
    }
    let name = output.file_name().expect("checked output name");
    let mut backup_name = OsString::from(".");
    backup_name.push(name);
    backup_name.push(format!(".bak-{}", std::process::id()));
    let backup = usable_parent(output).join(backup_name);
    if backup.exists() {
        return Err(format!("事务备份已存在: {}", backup.display()));
    }
    fs::rename(output, &backup)
        .map_err(|error| format!("无法暂存已有输出 {}: {error}", output.display()))?;
    if let Err(error) = fs::rename(temporary, output) {
        let _ = fs::rename(&backup, output);
        return Err(format!("无法提交输出 {}: {error}", output.display()));
    }
    fs::remove_file(&backup).map_err(|error| {
        format!(
            "输出已提交，但无法删除事务备份 {}: {error}",
            backup.display()
        )
    })
}

fn interactive_loop(initial: Option<PathBuf>) -> Result<(), String> {
    let mut prefill = initial;
    loop {
        println!("\nFunnyBee 无前缀 TXT 一体化工具");
        println!("  1. 递归提取 FDI / DAT / 脚本文本");
        println!("  2. 从 TXT 递归工作区回封");
        println!("  3. 仅提取散脚本单文件");
        println!("  4. 仅注入散脚本单文件");
        println!("  0. 退出");
        let choice = prompt("选择", Some("1"))?;
        if choice == "0" {
            return Ok(());
        }
        let mode = match choice.as_str() {
            "1" => Mode::ExtractRecursive,
            "2" => Mode::PackRecursive,
            "3" => Mode::ExtractSingle,
            "4" => Mode::InjectSingle,
            _ => {
                eprintln!("无效选择");
                continue;
            }
        };
        let default_input = prefill
            .take()
            .map(|path| path.to_string_lossy().into_owned());
        let input = PathBuf::from(prompt("输入路径", default_input.as_deref())?);
        let translation = if matches!(mode, Mode::InjectSingle) {
            Some(PathBuf::from(prompt("翻译 TXT 路径", None)?))
        } else {
            None
        };
        let workspace = if matches!(mode, Mode::PackRecursive) {
            Some(PathBuf::from(prompt("递归工作区路径", None)?))
        } else {
            None
        };
        let adv_optional = matches!(mode, Mode::ExtractRecursive | Mode::PackRecursive);
        let adv_label = if adv_optional {
            "ADV.COM 路径（留空自动发现/使用工作区记录）"
        } else {
            "ADV.COM 路径"
        };
        let adv_text = prompt(adv_label, Some(""))?;
        let adv = if adv_text.is_empty() {
            None
        } else {
            Some(PathBuf::from(adv_text))
        };
        if !adv_optional && adv.is_none() {
            eprintln!("散脚本提取/注入必须提供 ADV.COM");
            continue;
        }
        let output_default = suggested_output(mode, &input)
            .to_string_lossy()
            .into_owned();
        let output = PathBuf::from(prompt("输出路径", Some(&output_default))?);
        let max_columns = if matches!(mode, Mode::PackRecursive | Mode::InjectSingle) {
            let value = prompt("最大字符数（留空不自动换行）", Some(""))?;
            if value.is_empty() {
                None
            } else {
                let parsed = value
                    .parse::<usize>()
                    .map_err(|_| format!("无效最大字符数: {value}"))?;
                if parsed == 0 {
                    eprintln!("最大字符数必须大于 0");
                    continue;
                }
                Some(parsed)
            }
        } else {
            None
        };
        let overwrite = output.exists() && prompt_yes_no("输出已存在，覆盖")?;
        println!("输入: {}", input.display());
        if let Some(path) = &translation {
            println!("翻译: {}", path.display());
        }
        if let Some(path) = &workspace {
            println!("工作区: {}", path.display());
        }
        if let Some(path) = &adv {
            println!("ADV: {}", path.display());
        } else {
            println!("ADV: 自动发现");
        }
        println!("输出: {}", output.display());
        if !prompt_yes_no("确认执行")? {
            println!("已取消");
            continue;
        }
        if let Err(error) = execute(&Invocation {
            mode,
            input,
            translation,
            workspace,
            adv,
            output,
            max_columns,
            overwrite,
        }) {
            eprintln!("错误: {error}");
        }
    }
}

fn suggested_output(mode: Mode, input: &Path) -> PathBuf {
    let parent = usable_parent(input);
    match mode {
        Mode::ExtractRecursive => {
            let base = input
                .file_stem()
                .or_else(|| input.file_name())
                .unwrap_or_else(|| OsStr::new("output"));
            let mut name = base.to_os_string();
            name.push("_txt_workspace");
            parent.join(name)
        }
        Mode::ExtractSingle => {
            let base = input
                .file_stem()
                .or_else(|| input.file_name())
                .unwrap_or_else(|| OsStr::new("output"));
            let mut name = base.to_os_string();
            name.push(".txt");
            parent.join(name)
        }
        Mode::PackRecursive | Mode::InjectSingle => {
            let base = if input.is_file() {
                input.file_stem()
            } else {
                input.file_name()
            }
            .unwrap_or_else(|| OsStr::new("output"));
            let mut name = base.to_os_string();
            name.push("_rebuilt");
            if input.is_file() {
                if let Some(extension) = input.extension() {
                    name.push(".");
                    name.push(extension);
                }
            }
            parent.join(name)
        }
    }
}

fn usable_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn prompt(label: &str, default: Option<&str>) -> Result<String, String> {
    match default {
        Some(value) if !value.is_empty() => print!("{label} [{value}]: "),
        _ => print!("{label}: "),
    }
    io::stdout()
        .flush()
        .map_err(|error| format!("无法刷新终端: {error}"))?;
    let mut value = String::new();
    if io::stdin()
        .read_line(&mut value)
        .map_err(|error| format!("无法读取终端输入: {error}"))?
        == 0
    {
        return Err("终端输入结束".to_string());
    }
    let value = value.trim().to_string();
    if value.is_empty() {
        if let Some(default) = default {
            return Ok(default.to_string());
        }
    }
    Ok(value)
}

fn prompt_yes_no(label: &str) -> Result<bool, String> {
    Ok(matches!(
        prompt(&format!("{label} [y/N]"), Some("N"))?
            .to_ascii_lowercase()
            .as_str(),
        "y" | "yes"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_recursive_unicode_paths() {
        let invocation = parse_args(vec![
            OsString::from("extract-recursive"),
            OsString::from("--input"),
            OsString::from("宇宙怪盗"),
            OsString::from("--output"),
            OsString::from("纯文本"),
        ])
        .unwrap();
        assert!(matches!(invocation.mode, Mode::ExtractRecursive));
        assert_eq!(invocation.input, PathBuf::from("宇宙怪盗"));
        assert_eq!(invocation.output, PathBuf::from("纯文本"));
    }

    #[test]
    fn single_extract_requires_adv() {
        let error = parse_args(vec![
            OsString::from("extract"),
            OsString::from("--input"),
            OsString::from("script.bin"),
            OsString::from("--output"),
            OsString::from("script.txt"),
        ])
        .unwrap_err();
        assert!(error.contains("--adv"));
    }

    #[test]
    fn parses_recursive_pack_and_single_inject() {
        let pack = parse_args(vec![
            OsString::from("pack-recursive"),
            OsString::from("--source"),
            OsString::from("原盘"),
            OsString::from("--workspace"),
            OsString::from("工作区"),
            OsString::from("--output"),
            OsString::from("回封"),
        ])
        .unwrap();
        assert!(matches!(pack.mode, Mode::PackRecursive));
        assert_eq!(pack.workspace, Some(PathBuf::from("工作区")));

        let inject = parse_args(vec![
            OsString::from("inject"),
            OsString::from("--input"),
            OsString::from("script.bin"),
            OsString::from("--translation"),
            OsString::from("script.txt"),
            OsString::from("--adv"),
            OsString::from("ADV.COM"),
            OsString::from("--output"),
            OsString::from("rebuilt.bin"),
            OsString::from("--max-columns"),
            OsString::from("28"),
        ])
        .unwrap();
        assert!(matches!(inject.mode, Mode::InjectSingle));
        assert_eq!(inject.max_columns, Some(28));
    }
}
