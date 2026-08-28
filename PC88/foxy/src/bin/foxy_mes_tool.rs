use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use encoding_rs::SHIFT_JIS;
use foxy_d88_tool::{font, mes};

const CANCEL_TOKEN: &str = ":cancel";

fn main() {
    if let Err(error) = run() {
        eprintln!("错误：{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [] => interactive_menu(None),
        [arg] if arg == "-h" || arg == "--help" => {
            print_help();
            Ok(())
        }
        [path] if !path.starts_with('-') && path != "extract" && path != "inject" => {
            interactive_menu(Some(PathBuf::from(path)))
        }
        [command, rest @ ..] if command == "extract" => {
            let options = parse_extract_args(rest)?;
            run_extract(&options.input, &options.output, options.overwrite)
        }
        [command, rest @ ..] if command == "inject" => {
            let options = parse_inject_args(rest)?;
            run_inject(
                &options.input,
                &options.translations,
                &options.output,
                options.mapping_used.as_deref(),
                options.wrap_columns,
                options.overwrite,
            )
        }
        _ => Err("参数无效；请使用 --help 查看用法".to_owned()),
    }
}

#[derive(Debug)]
struct ExtractOptions {
    input: PathBuf,
    output: PathBuf,
    overwrite: bool,
}

#[derive(Debug)]
struct InjectOptions {
    input: PathBuf,
    translations: PathBuf,
    output: PathBuf,
    mapping_used: Option<PathBuf>,
    wrap_columns: Option<usize>,
    overwrite: bool,
}

fn parse_extract_args(args: &[String]) -> Result<ExtractOptions, String> {
    let mut input = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--input" => input = Some(next_path(args, &mut index, "--input")?),
            "--output" => output = Some(next_path(args, &mut index, "--output")?),
            "--overwrite" => overwrite = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            unknown => return Err(format!("extract 不支持参数：{unknown}")),
        }
        index += 1;
    }

    Ok(ExtractOptions {
        input: input.ok_or_else(|| "extract 缺少 --input".to_owned())?,
        output: output.ok_or_else(|| "extract 缺少 --output".to_owned())?,
        overwrite,
    })
}

fn parse_inject_args(args: &[String]) -> Result<InjectOptions, String> {
    let mut input = None;
    let mut translations = None;
    let mut output = None;
    let mut mapping_used = None;
    let mut wrap_columns = None;
    let mut overwrite = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--input" => input = Some(next_path(args, &mut index, "--input")?),
            "--translations" => translations = Some(next_path(args, &mut index, "--translations")?),
            "--output" => output = Some(next_path(args, &mut index, "--output")?),
            "--mapping-used" | "--mapping" => {
                mapping_used = Some(next_path(args, &mut index, "--mapping-used")?)
            }
            "--wrap-columns" => {
                let value = next_value(args, &mut index, "--wrap-columns")?;
                let parsed = value
                    .parse::<usize>()
                    .map_err(|_| format!("--wrap-columns 必须是正整数，实际为：{value}"))?;
                if parsed == 0 {
                    return Err("--wrap-columns 必须大于 0".to_owned());
                }
                wrap_columns = Some(parsed);
            }
            "--overwrite" => overwrite = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            unknown => return Err(format!("inject 不支持参数：{unknown}")),
        }
        index += 1;
    }

    Ok(InjectOptions {
        input: input.ok_or_else(|| "inject 缺少 --input".to_owned())?,
        translations: translations.ok_or_else(|| "inject 缺少 --translations".to_owned())?,
        output: output.ok_or_else(|| "inject 缺少 --output".to_owned())?,
        mapping_used,
        wrap_columns,
        overwrite,
    })
}

fn next_path(args: &[String], index: &mut usize, option: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(next_value(args, index, option)?))
}

fn next_value<'a>(args: &'a [String], index: &mut usize, option: &str) -> Result<&'a str, String> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or_else(|| format!("{option} 缺少值"))
}

fn run_extract(input: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    let entry_count = mes::extract_workspace(input, output, overwrite)?;
    let document = mes::load_translation_document(output)?;
    println!(
        "提取完成：{} 个 MES 文件，{} 条可翻译文本\n输出：{}",
        document.files.len(),
        entry_count,
        output.display()
    );
    Ok(())
}

fn run_inject(
    input: &Path,
    translations: &Path,
    output: &Path,
    mapping_used: Option<&Path>,
    wrap_columns: Option<usize>,
    overwrite: bool,
) -> Result<(), String> {
    let encoder = TextEncoder::load(mapping_used)?;
    let summary = mes::inject_workspace(
        input,
        translations,
        output,
        overwrite,
        wrap_columns,
        |text| encoder.encode(text),
    )?;

    println!(
        "注入完成：处理 {} 个 MES 文件，写入 {} 条已修改文本\n输出：{}\n报告：{}",
        summary.file_count,
        summary.changed_entry_count,
        output.display(),
        summary.report_path.display()
    );
    Ok(())
}

enum TextEncoder {
    Cp932,
    Dynamic {
        resources: font::FontResources,
        plan: font::DynamicFontPlan,
    },
}

impl TextEncoder {
    fn load(mapping_used: Option<&Path>) -> Result<Self, String> {
        match mapping_used {
            Some(path) => Ok(Self::Dynamic {
                resources: font::FontResources::load_embedded()?,
                plan: font::load_mapping_used(path)?,
            }),
            None => Ok(Self::Cp932),
        }
    }

    fn encode(&self, text: &str) -> Result<Vec<u8>, String> {
        let bytes = match self {
            Self::Cp932 => encode_cp932_text(text)?,
            Self::Dynamic { resources, plan } => resources.encode_ai1_text(text, plan)?.bytes,
        };
        validate_ai1_display_bytes(&bytes, text)?;
        Ok(bytes)
    }
}

fn encode_cp932_text(text: &str) -> Result<Vec<u8>, String> {
    let (encoded, _, had_errors) = SHIFT_JIS.encode(text);
    if had_errors {
        return Err(format!(
            "文本包含 CP932 无法编码的字符；中文注入请指定 --mapping-used：{text:?}"
        ));
    }
    Ok(encoded.into_owned())
}

fn validate_ai1_display_bytes(bytes: &[u8], text: &str) -> Result<(), String> {
    if bytes.is_empty() {
        return Ok(());
    }
    let pairs = bytes.chunks_exact(2);
    if !pairs.remainder().is_empty() || !pairs.clone().all(|pair| (0x80..=0x98).contains(&pair[0]))
    {
        return Err(format!(
            "AI1 文本只能使用脚本支持的双字节字形，无法注入：{text:?}"
        ));
    }
    Ok(())
}

#[derive(Default)]
struct InteractiveState {
    source: Option<PathBuf>,
    extract_output: Option<PathBuf>,
    translations: Option<PathBuf>,
    inject_output: Option<PathBuf>,
    mapping_used: Option<PathBuf>,
    wrap_columns: Option<usize>,
    overwrite: bool,
}

enum MenuFlow {
    Return,
    Exit,
}

enum Prompt<T> {
    Value(T),
    Cancel,
    Eof,
}

fn interactive_menu(prefill: Option<PathBuf>) -> Result<(), String> {
    let mut state = InteractiveState {
        source: prefill,
        ..InteractiveState::default()
    };

    loop {
        println!("\nFOXY MES 文本工具");
        println!("1  提取文本");
        println!("2  注入文本");
        println!("0  退出");
        print!("> ");
        io::stdout()
            .flush()
            .map_err(|error| format!("刷新输出失败：{error}"))?;

        let mut line = String::new();
        if io::stdin()
            .read_line(&mut line)
            .map_err(|error| format!("读取输入失败：{error}"))?
            == 0
        {
            return Ok(());
        }

        let result = match line.trim() {
            "1" => interactive_extract(&mut state),
            "2" => interactive_inject(&mut state),
            "0" => return Ok(()),
            _ => {
                println!("请输入 0、1 或 2。");
                continue;
            }
        };

        match result {
            Ok(MenuFlow::Return) => {}
            Ok(MenuFlow::Exit) => return Ok(()),
            Err(error) => eprintln!("操作失败：{error}"),
        }
    }
}

fn interactive_extract(state: &mut InteractiveState) -> Result<MenuFlow, String> {
    println!("\n提取文本（任一输入项键入 {CANCEL_TOKEN} 可取消）");
    let source = match prompt_required_path("D88 解包目录", state.source.as_deref())? {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };
    state.source = Some(source.clone());

    let suggested_output = state
        .extract_output
        .clone()
        .unwrap_or_else(|| suggested_sibling(&source, "text"));
    let output = match prompt_required_path("文本输出目录", Some(&suggested_output))? {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };
    let overwrite = match prompt_yes_no("覆盖已有的受管输出", state.overwrite)? {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };

    println!("\n即将提取：");
    println!("  源目录：{}", source.display());
    println!("  输出目录：{}", output.display());
    println!("  覆盖：{}", yes_no(overwrite));
    let confirmed = match prompt_yes_no("确认开始写入", false)? {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };
    if !confirmed {
        return cancelled();
    }

    run_extract(&source, &output, overwrite)?;
    state.extract_output = Some(output);
    state.overwrite = overwrite;
    Ok(MenuFlow::Return)
}

fn interactive_inject(state: &mut InteractiveState) -> Result<MenuFlow, String> {
    println!("\n注入文本（任一输入项键入 {CANCEL_TOKEN} 可取消）");
    let source = match prompt_required_path("源工作区（D88 解包目录）", state.source.as_deref())?
    {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };
    state.source = Some(source.clone());

    let suggested_translations = state
        .translations
        .clone()
        .or_else(|| state.extract_output.clone())
        .unwrap_or_else(|| suggested_sibling(&source, "text"));
    let translations =
        match prompt_required_path("翻译 JSON 文件或文本工作区", Some(&suggested_translations))?
        {
            Prompt::Value(value) => value,
            Prompt::Cancel => return cancelled(),
            Prompt::Eof => return Ok(MenuFlow::Exit),
        };

    let suggested_output = state
        .inject_output
        .clone()
        .unwrap_or_else(|| suggested_sibling(&source, "localized"));
    let output = match prompt_required_path("本地化文件输出目录", Some(&suggested_output))?
    {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };
    let mapping_used = match prompt_optional_path(
        "mapping_used.json（仅 CP932 文本可留空，输入 - 可清除）",
        state.mapping_used.as_deref(),
    )? {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };
    let wrap_columns = match prompt_wrap_columns(state.wrap_columns)? {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };
    let overwrite = match prompt_yes_no("覆盖已有的受管输出", state.overwrite)? {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };

    println!("\n即将注入：");
    println!("  源工作区：{}", source.display());
    println!("  翻译：{}", translations.display());
    println!("  输出目录：{}", output.display());
    println!(
        "  动态映射：{}",
        mapping_used.as_deref().map_or_else(
            || "不使用（仅 CP932）".to_owned(),
            |path| path.display().to_string()
        )
    );
    println!(
        "  自动插入换行：{}",
        wrap_columns.map_or_else(|| "关闭".to_owned(), |value| format!("每 {value} 列"))
    );
    println!("  覆盖：{}", yes_no(overwrite));
    let confirmed = match prompt_yes_no("确认开始写入", false)? {
        Prompt::Value(value) => value,
        Prompt::Cancel => return cancelled(),
        Prompt::Eof => return Ok(MenuFlow::Exit),
    };
    if !confirmed {
        return cancelled();
    }

    run_inject(
        &source,
        &translations,
        &output,
        mapping_used.as_deref(),
        wrap_columns,
        overwrite,
    )?;
    state.translations = Some(translations);
    state.inject_output = Some(output);
    state.mapping_used = mapping_used;
    state.wrap_columns = wrap_columns;
    state.overwrite = overwrite;
    Ok(MenuFlow::Return)
}

fn prompt_required_path(label: &str, default: Option<&Path>) -> Result<Prompt<PathBuf>, String> {
    loop {
        let default_text = default.map(|path| path.display().to_string());
        match read_prompt(label, default_text.as_deref())? {
            Prompt::Value(value) if value.trim().is_empty() => {
                if let Some(default) = default {
                    return Ok(Prompt::Value(default.to_path_buf()));
                }
                println!("此项不能为空。");
            }
            Prompt::Value(value) => return Ok(Prompt::Value(PathBuf::from(value.trim()))),
            Prompt::Cancel => return Ok(Prompt::Cancel),
            Prompt::Eof => return Ok(Prompt::Eof),
        }
    }
}

fn prompt_optional_path(
    label: &str,
    default: Option<&Path>,
) -> Result<Prompt<Option<PathBuf>>, String> {
    let default_text = default.map(|path| path.display().to_string());
    match read_prompt(label, default_text.as_deref())? {
        Prompt::Value(value) if value.trim().is_empty() => {
            Ok(Prompt::Value(default.map(Path::to_path_buf)))
        }
        Prompt::Value(value)
            if value.trim() == "-" || value.trim().eq_ignore_ascii_case("none") =>
        {
            Ok(Prompt::Value(None))
        }
        Prompt::Value(value) => Ok(Prompt::Value(Some(PathBuf::from(value.trim())))),
        Prompt::Cancel => Ok(Prompt::Cancel),
        Prompt::Eof => Ok(Prompt::Eof),
    }
}

fn prompt_wrap_columns(default: Option<usize>) -> Result<Prompt<Option<usize>>, String> {
    loop {
        let default_text = default.map(|value| value.to_string());
        match read_prompt(
            "自动换行列数（留空保持，输入 0 或 none 关闭）",
            default_text.as_deref(),
        )? {
            Prompt::Value(value) if value.trim().is_empty() => {
                return Ok(Prompt::Value(default));
            }
            Prompt::Value(value)
                if value.trim() == "0" || value.trim().eq_ignore_ascii_case("none") =>
            {
                return Ok(Prompt::Value(None));
            }
            Prompt::Value(value) => match value.trim().parse::<usize>() {
                Ok(parsed) if parsed > 0 => return Ok(Prompt::Value(Some(parsed))),
                _ => println!("请输入正整数，或输入 0/none 关闭。"),
            },
            Prompt::Cancel => return Ok(Prompt::Cancel),
            Prompt::Eof => return Ok(Prompt::Eof),
        }
    }
}

fn prompt_yes_no(label: &str, default: bool) -> Result<Prompt<bool>, String> {
    loop {
        let hint = if default { "Y/n" } else { "y/N" };
        match read_prompt(label, Some(hint))? {
            Prompt::Value(value) if value.trim().is_empty() => {
                return Ok(Prompt::Value(default));
            }
            Prompt::Value(value) => match value.trim().to_ascii_lowercase().as_str() {
                "y" | "yes" | "1" => return Ok(Prompt::Value(true)),
                "n" | "no" | "0" => return Ok(Prompt::Value(false)),
                _ => println!("请输入 y 或 n。"),
            },
            Prompt::Cancel => return Ok(Prompt::Cancel),
            Prompt::Eof => return Ok(Prompt::Eof),
        }
    }
}

fn read_prompt(label: &str, default: Option<&str>) -> Result<Prompt<String>, String> {
    match default {
        Some(value) => print!("{label} [{value}]："),
        None => print!("{label}："),
    }
    io::stdout()
        .flush()
        .map_err(|error| format!("刷新输出失败：{error}"))?;

    let mut line = String::new();
    if io::stdin()
        .read_line(&mut line)
        .map_err(|error| format!("读取输入失败：{error}"))?
        == 0
    {
        return Ok(Prompt::Eof);
    }
    let value = line.trim_end_matches(['\r', '\n']).to_owned();
    if value.trim().eq_ignore_ascii_case(CANCEL_TOKEN) {
        Ok(Prompt::Cancel)
    } else {
        Ok(Prompt::Value(value))
    }
}

fn suggested_sibling(source: &Path, suffix: &str) -> PathBuf {
    let base = source
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("foxy");
    let name = format!("{base}_{suffix}");
    source.parent().unwrap_or_else(|| Path::new(".")).join(name)
}

fn cancelled() -> Result<MenuFlow, String> {
    println!("操作已取消，未写入文件。");
    Ok(MenuFlow::Return)
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "是"
    } else {
        "否"
    }
}

fn print_help() {
    println!(
        "FOXY MES 文本提取/注入工具\n\n\
用法：\n\
  foxy_mes_tool\n\
  foxy_mes_tool <D88解包目录>\n\
  foxy_mes_tool extract --input <解包目录> --output <文本目录> [--overwrite]\n\
  foxy_mes_tool inject --input <解包目录> --translations <JSON或文本目录> \\\n      --output <本地化目录> [--mapping-used <mapping_used.json>] \\\n      [--wrap-columns <正整数>] [--overwrite]\n\n\
无参数或只给一个路径会进入持续菜单，所有路径和选项都可在写入前修改并确认。\n\
默认不自动插入换行；只有指定 --wrap-columns 才会插入 AI1 显式换行 81 93。\n\
中文注入必须使用字体构建程序生成的 mapping_used.json；不指定映射时仅支持 CP932。"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cp932_ai1_text_requires_double_byte_glyphs() {
        assert!(TextEncoder::Cp932.encode("屋上").is_ok());
        assert!(TextEncoder::Cp932.encode("ABC").is_err());
    }

    #[test]
    fn suggested_output_is_editable_sibling_default() {
        let input = Path::new(r"C:\game\disk_unpacked");
        assert_eq!(
            suggested_sibling(input, "text"),
            PathBuf::from(r"C:\game\disk_unpacked_text")
        );
    }
}
