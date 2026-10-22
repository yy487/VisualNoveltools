use funnybee_script_tool::{
    analyze_file, extract_translation, inject_translation, recursive_extract, recursive_pack,
    GlyphTable, InjectOptions, RecursivePackOptions, TranslationDocument,
};
use serde::Serialize;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"funnybee_tool 0.3.2

《宇宙怪盗ファニーBee》递归 FDI / DRI DAT / System 3 文本工具。

用法:
  funnybee_tool.exe extract-recursive --input <FILE_OR_DIR> --output <WORKSPACE> [--adv <ADV.COM>]
  funnybee_tool.exe pack-recursive --source <FILE_OR_DIR> --workspace <WORKSPACE> --output <FILE_OR_DIR> [--adv <ADV.COM>] [--max-columns N]
  funnybee_tool.exe extract --input <MEMBER.BIN> --adv <ADV.COM> --output <FILE.JSON>
  funnybee_tool.exe extract-dir --input <MEMBERS_DIR> --adv <ADV.COM> --output <JSON_DIR>
  funnybee_tool.exe inject --input <MEMBER.BIN> --translation <FILE.JSON> --adv <ADV.COM> --output <MEMBER.BIN> [--max-columns N]
  funnybee_tool.exe inject-dir --input <MEMBERS_DIR> --translation <JSON_DIR> --adv <ADV.COM> --output <MEMBERS_DIR> [--max-columns N]
  funnybee_tool.exe analyze --input <MEMBER.BIN> --adv <ADV.COM> --output <FILE.JSON>
  funnybee_tool.exe analyze-dir --input <MEMBERS_DIR> --adv <ADV.COM> --output <OUTPUT_DIR>

选项:
  --overwrite       明确允许覆盖已有输出
  --max-columns N   注入时按最多 N 个静态字符插入 0x52；含 H/X 的动态行不自动断行

递归提取按内容识别 FDI、DRI DAT 和脚本；递归回封会复制完整源树并替换修改项。
不带参数或只给一个路径时进入交互菜单，确认前不写文件。
"#;

#[derive(Clone, Copy, Debug)]
enum Mode {
    AnalyzeFile,
    AnalyzeDirectory,
    ExtractFile,
    ExtractDirectory,
    InjectFile,
    InjectDirectory,
    ExtractRecursive,
    PackRecursive,
}

#[derive(Clone, Debug)]
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

#[derive(Serialize)]
struct DirectoryManifest {
    _format: String,
    source_dir: String,
    adv_file: String,
    files: Vec<DirectoryEntry>,
}

#[derive(Serialize)]
struct DirectoryEntry {
    source_file: String,
    output_file: String,
    strings: usize,
    source_bytes: usize,
}

#[derive(Default)]
struct DirectoryInjectSummary {
    files: usize,
    entries: usize,
    changed_entries: usize,
    inserted_newlines: usize,
    dynamic_wrap_skips: usize,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("错误: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
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
        Some("analyze") => Mode::AnalyzeFile,
        Some("analyze-dir") => Mode::AnalyzeDirectory,
        Some("extract") => Mode::ExtractFile,
        Some("extract-dir") => Mode::ExtractDirectory,
        Some("inject") => Mode::InjectFile,
        Some("inject-dir") => Mode::InjectDirectory,
        Some("extract-recursive") => Mode::ExtractRecursive,
        Some("pack-recursive") => Mode::PackRecursive,
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
            .ok_or_else(|| "命令选项无法表示为 Unicode".to_string())?;
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
    let needs_translation = matches!(mode, Mode::InjectFile | Mode::InjectDirectory);
    let needs_workspace = matches!(mode, Mode::PackRecursive);
    let recursive = matches!(mode, Mode::ExtractRecursive | Mode::PackRecursive);
    if !needs_translation && translation.is_some() {
        return Err("只有 inject/inject-dir 接受 --translation".to_string());
    }
    if !needs_workspace && workspace.is_some() {
        return Err("只有 pack-recursive 接受 --workspace".to_string());
    }
    if !matches!(
        mode,
        Mode::InjectFile | Mode::InjectDirectory | Mode::PackRecursive
    ) && max_columns.is_some()
    {
        return Err("只有 inject/inject-dir/pack-recursive 接受 --max-columns".to_string());
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
        adv: if recursive {
            adv
        } else {
            Some(adv.ok_or_else(|| "缺少 --adv".to_string())?)
        },
        output: output.ok_or_else(|| "缺少 --output".to_string())?,
        max_columns,
        overwrite,
    })
}

fn set_path(slot: &mut Option<PathBuf>, value: &OsString, option: &str) -> Result<(), String> {
    if slot.replace(PathBuf::from(value)).is_some() {
        return Err(format!("{option} 只能指定一次"));
    }
    Ok(())
}

fn execute(invocation: &Invocation) -> Result<(), String> {
    match invocation.mode {
        Mode::AnalyzeFile => analyze_one(
            &invocation.input,
            invocation.adv.as_deref().expect("adv"),
            &invocation.output,
            invocation.overwrite,
        ),
        Mode::AnalyzeDirectory => analyze_directory(
            &invocation.input,
            invocation.adv.as_deref().expect("adv"),
            &invocation.output,
            invocation.overwrite,
        ),
        Mode::ExtractFile => extract_one(
            &invocation.input,
            invocation.adv.as_deref().expect("adv"),
            &invocation.output,
            invocation.overwrite,
        ),
        Mode::ExtractDirectory => extract_directory(
            &invocation.input,
            invocation.adv.as_deref().expect("adv"),
            &invocation.output,
            invocation.overwrite,
        ),
        Mode::InjectFile => inject_one(
            &invocation.input,
            invocation.translation.as_deref().expect("translation"),
            invocation.adv.as_deref().expect("adv"),
            &invocation.output,
            invocation.max_columns,
            invocation.overwrite,
        ),
        Mode::InjectDirectory => inject_directory(
            &invocation.input,
            invocation.translation.as_deref().expect("translation"),
            invocation.adv.as_deref().expect("adv"),
            &invocation.output,
            invocation.max_columns,
            invocation.overwrite,
        ),
        Mode::ExtractRecursive => run_recursive_extract(invocation),
        Mode::PackRecursive => run_recursive_pack(invocation),
    }
}

fn run_recursive_extract(invocation: &Invocation) -> Result<(), String> {
    let report = recursive_extract(
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
    for dat in &report.dat_outputs {
        println!("DAT: {dat}");
    }
    for warning in report.warnings {
        eprintln!("警告: {warning}");
    }
    Ok(())
}

fn run_recursive_pack(invocation: &Invocation) -> Result<(), String> {
    let report = recursive_pack(
        &invocation.input,
        invocation.workspace.as_deref().expect("workspace"),
        invocation.adv.as_deref(),
        &invocation.output,
        RecursivePackOptions {
            max_columns: invocation.max_columns,
            overwrite: invocation.overwrite,
        },
    )?;
    println!(
        "完成: containers={} scripts={} changed_scripts={} changed_dat={} changed_fdi={} output={}",
        report.containers,
        report.scripts,
        report.changed_scripts,
        report.changed_dat_archives,
        report.changed_fdi_images,
        report.output.display()
    );
    Ok(())
}

fn analyze_one(input: &Path, adv: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    ensure_file_output(output, overwrite)?;
    let analysis = analyze_file(input, adv)?;
    write_json(output, &analysis)?;
    println!(
        "完成: strings={} parsed={}/{} output={}",
        analysis.strings.len(),
        analysis.parsed_bytes,
        analysis.source_bytes,
        output.display()
    );
    Ok(())
}

fn extract_one(input: &Path, adv: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    ensure_file_output(output, overwrite)?;
    let (source, glyphs, source_name) = load_source(input, adv)?;
    let document = extract_translation(&source, &glyphs, &source_name)?;
    write_json(output, &document)?;
    println!(
        "完成: entries={} output={}",
        document.entries.len(),
        output.display()
    );
    Ok(())
}

fn inject_one(
    input: &Path,
    translation: &Path,
    adv: &Path,
    output: &Path,
    max_columns: Option<usize>,
    overwrite: bool,
) -> Result<(), String> {
    ensure_file_output(output, overwrite)?;
    let (source, glyphs, source_name) = load_source(input, adv)?;
    let document: TranslationDocument = read_json(translation)?;
    let report = inject_translation(
        &source,
        &glyphs,
        &source_name,
        &document,
        InjectOptions { max_columns },
    )?;
    write_bytes(output, &report.bytes)?;
    println!(
        "完成: entries={} changed={} wraps={} dynamic_wrap_skips={} bytes={} output={}",
        report.entries,
        report.changed_entries,
        report.inserted_newlines,
        report.dynamic_wrap_skips,
        report.bytes.len(),
        output.display()
    );
    Ok(())
}

fn analyze_directory(
    input: &Path,
    adv: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(), String> {
    directory_json_operation(
        input,
        adv,
        output,
        overwrite,
        "funnybee-system3-script-analysis-directory-v1",
        |source, adv| analyze_file(source, adv).and_then(to_json_value),
        |value| {
            value
                .get("strings")
                .and_then(|value| value.as_array())
                .map_or(0, Vec::len)
        },
    )
}

fn extract_directory(
    input: &Path,
    adv: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(), String> {
    let adv_bytes =
        fs::read(adv).map_err(|error| format!("无法读取 {}: {error}", adv.display()))?;
    let glyphs = GlyphTable::from_adv(&adv_bytes)?;
    directory_json_operation(
        input,
        adv,
        output,
        overwrite,
        "funnybee-system3-translation-directory-v1",
        |source, _| {
            let bytes = fs::read(source)
                .map_err(|error| format!("无法读取 {}: {error}", source.display()))?;
            let name = unicode_file_name(source)?;
            extract_translation(&bytes, &glyphs, &name).and_then(to_json_value)
        },
        |value| {
            value
                .get("entries")
                .and_then(|value| value.as_array())
                .map_or(0, Vec::len)
        },
    )
}

fn directory_json_operation<F, C>(
    input: &Path,
    adv: &Path,
    output: &Path,
    overwrite: bool,
    format: &str,
    mut convert: F,
    count_strings: C,
) -> Result<(), String>
where
    F: FnMut(&Path, &Path) -> Result<serde_json::Value, String>,
    C: Fn(&serde_json::Value) -> usize,
{
    let files = list_files(input)?;
    prepare_directory_output(output, overwrite)?;
    let staging = staging_path(output)?;
    fs::create_dir_all(&staging)
        .map_err(|error| format!("无法建立 {}: {error}", staging.display()))?;
    let result = (|| {
        let mut entries = Vec::with_capacity(files.len());
        for source in &files {
            let file_name = unicode_file_name(source)?;
            let output_name = format!("{file_name}.json");
            let value =
                convert(source, adv).map_err(|error| format!("{}: {error}", source.display()))?;
            write_json(&staging.join(&output_name), &value)?;
            entries.push(DirectoryEntry {
                source_file: file_name,
                output_file: output_name,
                strings: count_strings(&value),
                source_bytes: source
                    .metadata()
                    .map_err(|error| format!("无法读取文件属性 {}: {error}", source.display()))?
                    .len() as usize,
            });
        }
        let manifest = DirectoryManifest {
            _format: format.to_string(),
            source_dir: path_name(input, "输入目录")?,
            adv_file: path_name(adv, "ADV 文件")?,
            files: entries,
        };
        write_json(&staging.join("workspace.json"), &manifest)?;
        Ok::<DirectoryManifest, String>(manifest)
    })();
    match result {
        Ok(manifest) => {
            commit_directory(&staging, output, overwrite)?;
            let strings: usize = manifest.files.iter().map(|entry| entry.strings).sum();
            println!(
                "完成: files={} entries={} output={}",
                manifest.files.len(),
                strings,
                output.display()
            );
            Ok(())
        }
        Err(error) => {
            cleanup_staging(&staging);
            Err(error)
        }
    }
}

fn inject_directory(
    input: &Path,
    translation: &Path,
    adv: &Path,
    output: &Path,
    max_columns: Option<usize>,
    overwrite: bool,
) -> Result<(), String> {
    if !translation.is_dir() {
        return Err(format!("翻译输入不是目录: {}", translation.display()));
    }
    let files = list_files(input)?;
    let adv_bytes =
        fs::read(adv).map_err(|error| format!("无法读取 {}: {error}", adv.display()))?;
    let glyphs = GlyphTable::from_adv(&adv_bytes)?;
    prepare_directory_output(output, overwrite)?;
    let staging = staging_path(output)?;
    fs::create_dir_all(&staging)
        .map_err(|error| format!("无法建立 {}: {error}", staging.display()))?;
    let result = (|| {
        let mut summary = DirectoryInjectSummary::default();
        for source_path in &files {
            let source_name = unicode_file_name(source_path)?;
            let translation_path = translation.join(format!("{source_name}.json"));
            let source = fs::read(source_path)
                .map_err(|error| format!("无法读取 {}: {error}", source_path.display()))?;
            let document: TranslationDocument = read_json(&translation_path)?;
            let report = inject_translation(
                &source,
                &glyphs,
                &source_name,
                &document,
                InjectOptions { max_columns },
            )
            .map_err(|error| format!("{}: {error}", source_path.display()))?;
            write_bytes(&staging.join(&source_name), &report.bytes)?;
            summary.files += 1;
            summary.entries += report.entries;
            summary.changed_entries += report.changed_entries;
            summary.inserted_newlines += report.inserted_newlines;
            summary.dynamic_wrap_skips += report.dynamic_wrap_skips;
        }
        Ok::<DirectoryInjectSummary, String>(summary)
    })();
    match result {
        Ok(summary) => {
            commit_directory(&staging, output, overwrite)?;
            println!(
                "完成: files={} entries={} changed={} wraps={} dynamic_wrap_skips={} output={}",
                summary.files,
                summary.entries,
                summary.changed_entries,
                summary.inserted_newlines,
                summary.dynamic_wrap_skips,
                output.display()
            );
            Ok(())
        }
        Err(error) => {
            cleanup_staging(&staging);
            Err(error)
        }
    }
}

fn list_files(input: &Path) -> Result<Vec<PathBuf>, String> {
    if !input.is_dir() {
        return Err(format!("输入不是目录: {}", input.display()));
    }
    let mut files = fs::read_dir(input)
        .map_err(|error| format!("无法读取目录 {}: {error}", input.display()))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| format!("无法枚举目录 {}: {error}", input.display()))?;
    files.retain(|path| path.is_file());
    files.sort();
    if files.is_empty() {
        return Err(format!("输入目录没有文件: {}", input.display()));
    }
    Ok(files)
}

fn load_source(input: &Path, adv: &Path) -> Result<(Vec<u8>, GlyphTable, String), String> {
    let source =
        fs::read(input).map_err(|error| format!("无法读取 {}: {error}", input.display()))?;
    let adv_bytes =
        fs::read(adv).map_err(|error| format!("无法读取 {}: {error}", adv.display()))?;
    let glyphs = GlyphTable::from_adv(&adv_bytes)?;
    Ok((source, glyphs, unicode_file_name(input)?))
}

fn unicode_file_name(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .ok_or_else(|| format!("文件名无法表示为 Unicode: {}", path.display()))
}

fn path_name(path: &Path, role: &str) -> Result<String, String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .ok_or_else(|| format!("{role}名称无法表示为 Unicode: {}", path.display()))
}

fn to_json_value<T: Serialize>(value: T) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|error| format!("无法生成 JSON: {error}"))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("无法读取 JSON {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("无法解析 UTF-8 JSON {}: {error}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("无法序列化 {}: {error}", path.display()))?;
    bytes.push(b'\n');
    write_bytes(path, &bytes)
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法建立 {}: {error}", parent.display()))?;
    }
    let name = path
        .file_name()
        .ok_or_else(|| format!("输出不能是文件系统根: {}", path.display()))?;
    let mut temporary_name = name.to_os_string();
    temporary_name.push(".tmp");
    let temporary = path.with_file_name(temporary_name);
    if temporary.exists() {
        return Err(format!("临时文件已存在，拒绝使用: {}", temporary.display()));
    }
    fs::write(&temporary, bytes)
        .map_err(|error| format!("无法写入临时文件 {}: {error}", temporary.display()))?;
    if path.exists() {
        if let Err(error) = fs::remove_file(path) {
            let _ = fs::remove_file(&temporary);
            return Err(format!("无法替换已有文件 {}: {error}", path.display()));
        }
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "无法提交输出 {} -> {}: {error}",
            temporary.display(),
            path.display()
        ));
    }
    Ok(())
}

fn ensure_file_output(output: &Path, overwrite: bool) -> Result<(), String> {
    if output.exists() && !overwrite {
        return Err(format!(
            "输出已存在，需明确加 --overwrite: {}",
            output.display()
        ));
    }
    if output.is_dir() {
        return Err(format!("文件输出路径是目录: {}", output.display()));
    }
    Ok(())
}

fn prepare_directory_output(output: &Path, overwrite: bool) -> Result<(), String> {
    if output.exists() && !overwrite {
        return Err(format!(
            "输出目录已存在，需明确加 --overwrite: {}",
            output.display()
        ));
    }
    if output.exists() && !output.is_dir() {
        return Err(format!("目录输出路径是文件: {}", output.display()));
    }
    let staging = staging_path(output)?;
    if staging.exists() {
        return Err(format!("临时输出已存在，拒绝使用: {}", staging.display()));
    }
    Ok(())
}

fn staging_path(output: &Path) -> Result<PathBuf, String> {
    let name = output
        .file_name()
        .ok_or_else(|| format!("输出目录不能是文件系统根: {}", output.display()))?;
    let mut staging_name = name.to_os_string();
    staging_name.push(".tmp");
    Ok(output.with_file_name(staging_name))
}

fn commit_directory(staging: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    if output.exists() {
        if !overwrite {
            return Err(format!("输出目录已存在: {}", output.display()));
        }
        fs::remove_dir_all(output)
            .map_err(|error| format!("无法替换已有输出 {}: {error}", output.display()))?;
    }
    fs::rename(staging, output).map_err(|error| {
        format!(
            "无法提交输出 {} -> {}: {error}",
            staging.display(),
            output.display()
        )
    })
}

fn cleanup_staging(staging: &Path) {
    if staging
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.ends_with(".tmp"))
    {
        let _ = fs::remove_dir_all(staging);
    }
}

fn interactive_loop(initial: Option<PathBuf>) -> Result<(), String> {
    let mut prefill = initial;
    loop {
        println!("\nFunnyBee 递归提取/回封工具");
        println!("  1. 递归提取 FDI / DAT / 脚本文本");
        println!("  2. 从递归工作区回封");
        println!("  3. 仅提取散脚本单文件");
        println!("  4. 仅提取散脚本目录");
        println!("  5. 仅注入散脚本单文件");
        println!("  6. 仅注入散脚本目录");
        println!("  0. 退出");
        let choice = prompt("选择", Some("1"))?;
        if choice == "0" {
            return Ok(());
        }
        let mode = match choice.as_str() {
            "1" => Mode::ExtractRecursive,
            "2" => Mode::PackRecursive,
            "3" => Mode::ExtractFile,
            "4" => Mode::ExtractDirectory,
            "5" => Mode::InjectFile,
            "6" => Mode::InjectDirectory,
            _ => {
                eprintln!("无效选择");
                continue;
            }
        };
        let default_input = prefill
            .take()
            .map(|path| path.to_string_lossy().into_owned());
        let input = PathBuf::from(prompt("输入路径", default_input.as_deref())?);
        let translation = if matches!(mode, Mode::InjectFile | Mode::InjectDirectory) {
            Some(PathBuf::from(prompt("翻译 JSON 路径", None)?))
        } else {
            None
        };
        let workspace = if matches!(mode, Mode::PackRecursive) {
            Some(PathBuf::from(prompt("递归工作区路径", None)?))
        } else {
            None
        };
        let adv_optional = matches!(mode, Mode::ExtractRecursive | Mode::PackRecursive);
        let adv_text = if adv_optional {
            prompt("ADV.COM 路径（留空自动发现/使用工作区记录）", Some(""))?
        } else {
            prompt("ADV.COM 路径", None)?
        };
        let adv = if adv_text.is_empty() {
            None
        } else {
            Some(PathBuf::from(adv_text))
        };
        let output_default = suggested_output_path(mode, &input)
            .to_string_lossy()
            .into_owned();
        let output = PathBuf::from(prompt("输出路径", Some(&output_default))?);
        let max_columns = if matches!(
            mode,
            Mode::InjectFile | Mode::InjectDirectory | Mode::PackRecursive
        ) {
            let value = prompt("最大字符数（留空不自动换行）", Some(""))?;
            if value.is_empty() {
                None
            } else {
                Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("无效最大字符数: {value}"))?,
                )
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
            println!("ADV: 自动发现/工作区记录");
        }
        println!("输出: {}", output.display());
        if !prompt_yes_no("确认执行")? {
            println!("已取消");
            continue;
        }
        let invocation = Invocation {
            mode,
            input,
            translation,
            workspace,
            adv,
            output,
            max_columns,
            overwrite,
        };
        if let Err(error) = execute(&invocation) {
            eprintln!("错误: {error}");
        }
    }
}

fn suggested_output_path(mode: Mode, input: &Path) -> PathBuf {
    suggested_output_path_for_kind(mode, input, input.is_file())
}

fn suggested_output_path_for_kind(mode: Mode, input: &Path, input_is_file: bool) -> PathBuf {
    match mode {
        Mode::AnalyzeFile => sibling_file_with_suffix(input, "_analysis", true),
        Mode::AnalyzeDirectory => sibling_directory_with_suffix(input, "_analysis", false),
        Mode::ExtractFile => sibling_file_with_suffix(input, "", true),
        Mode::ExtractDirectory => sibling_directory_with_suffix(input, "_json", false),
        Mode::InjectFile => sibling_file_with_suffix(input, "_rebuilt", false),
        Mode::InjectDirectory => sibling_directory_with_suffix(input, "_rebuilt", false),
        Mode::ExtractRecursive => sibling_directory_with_suffix(input, "_workspace", input_is_file),
        Mode::PackRecursive if input_is_file => sibling_file_with_suffix(input, "_rebuilt", false),
        Mode::PackRecursive => sibling_directory_with_suffix(input, "_rebuilt", false),
    }
}

fn sibling_directory_with_suffix(input: &Path, suffix: &str, strip_extension: bool) -> PathBuf {
    let parent = usable_parent(input);
    let base = if strip_extension {
        input.file_stem()
    } else {
        input.file_name()
    }
    .unwrap_or_else(|| OsStr::new("output"));
    let mut name = base.to_os_string();
    name.push(suffix);
    parent.join(name)
}

fn sibling_file_with_suffix(input: &Path, suffix: &str, json_extension: bool) -> PathBuf {
    let parent = usable_parent(input);
    let base = input
        .file_stem()
        .or_else(|| input.file_name())
        .unwrap_or_else(|| OsStr::new("output"));
    let mut name = base.to_os_string();
    name.push(suffix);
    let extension = if json_extension {
        Some(OsStr::new("json"))
    } else {
        input.extension()
    };
    if let Some(extension) = extension {
        name.push(".");
        name.push(extension);
    }
    parent.join(name)
}

fn usable_parent(input: &Path) -> &Path {
    input
        .parent()
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
    let count = io::stdin()
        .read_line(&mut value)
        .map_err(|error| format!("无法读取终端输入: {error}"))?;
    if count == 0 {
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
    fn recursive_extract_does_not_require_external_adv() {
        let invocation = parse_args(vec![
            OsString::from("extract-recursive"),
            OsString::from("--input"),
            OsString::from("input"),
            OsString::from("--output"),
            OsString::from("workspace"),
        ])
        .unwrap();
        assert!(matches!(invocation.mode, Mode::ExtractRecursive));
        assert!(invocation.adv.is_none());
    }

    #[test]
    fn recursive_pack_requires_workspace() {
        let error = parse_args(vec![
            OsString::from("pack-recursive"),
            OsString::from("--source"),
            OsString::from("input"),
            OsString::from("--output"),
            OsString::from("rebuilt"),
        ])
        .unwrap_err();
        assert!(error.contains("--workspace"));
    }

    #[test]
    fn interactive_recursive_extract_has_a_nonempty_sibling_default() {
        let input = Path::new(r"C:\游戏\1");
        let output = suggested_output_path_for_kind(Mode::ExtractRecursive, input, false);
        assert_eq!(output, PathBuf::from(r"C:\游戏\1_workspace"));
    }

    #[test]
    fn interactive_single_fdi_pack_preserves_extension() {
        let input = Path::new(r"C:\游戏\bee-a.fdi");
        let output = suggested_output_path_for_kind(Mode::PackRecursive, input, true);
        assert_eq!(output, PathBuf::from(r"C:\游戏\bee-a_rebuilt.fdi"));
    }
}
