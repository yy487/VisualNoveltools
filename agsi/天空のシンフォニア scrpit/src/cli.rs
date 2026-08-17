use crate::bundle::{Bundle, BundleError};
use crate::extract::{build_plan, extract_workspace, ExtractOptions};
use crate::inject::{inject_workspace, preview_inject, read_workspace, InjectOptions};
use crate::workspace::resolve_workspace_role;
use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn help() {
    println!(
        r#"AGSI Sinfonia script text tool

用法:
  sinfonia-script-tool extract --input <UNPACKED_DIR> --output <WORKSPACE_DIR> [--overwrite]
  sinfonia-script-tool inject --workspace <WORKSPACE_DIR> --output <INJECTED_DIR> [--source <SOURCE_DIR>] [--translations <TRANSLATION_DIR>] [--overwrite]
  sinfonia-script-tool

extract 读取 SB2 V2/兼容 SIMPLE 解包目录，创建带 source 快照和 UTF-8 JSON 的受管工作区。
inject 校验并应用工作区翻译，重建 CSTR，并生成可交给 SB2 工具封包的目录。

完整子命令不会询问输入；无参数或仅路径参数进入交互菜单，确认前不写文件。
输出已存在时默认拒绝覆盖，只有 --overwrite 或交互确认才覆盖。
路径可包含空格、&、日文和其他 Unicode 字符。"#
    );
}

fn trim_drag_path(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn option(args: &[String], names: &[&str]) -> Result<Option<String>, BundleError> {
    for (index, arg) in args.iter().enumerate() {
        if names.contains(&arg.as_str()) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{} 缺少值", arg))?;
            if value.starts_with('-') {
                return Err(format!("{} 缺少值", arg));
            }
            return Ok(Some(value.clone()));
        }
    }
    Ok(None)
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn reject_unknown_options(args: &[String], allowed: &[&str]) -> Result<(), BundleError> {
    let value_options = [
        "--input",
        "-i",
        "--output",
        "-o",
        "--workspace",
        "--source",
        "--translations",
    ];
    let mut seen = BTreeSet::new();
    let mut index = 0usize;
    while index < args.len() {
        let arg = &args[index];
        if !allowed.contains(&arg.as_str()) {
            return Err(format!("未知参数: {}", arg));
        }
        if !seen.insert(arg.as_str()) {
            return Err(format!("参数重复: {}", arg));
        }
        index += 1;
        if value_options.contains(&arg.as_str()) {
            let value = args.get(index).ok_or_else(|| format!("{} 缺少值", arg))?;
            if value.starts_with('-') {
                return Err(format!("{} 缺少值", arg));
            }
            index += 1;
        }
    }
    Ok(())
}

fn run_extract_cli(args: &[String]) -> Result<(), BundleError> {
    reject_unknown_options(
        args,
        &[
            "--input",
            "-i",
            "--output",
            "-o",
            "--overwrite",
            "--non-interactive",
        ],
    )?;
    let input =
        option(args, &["--input", "-i"])?.ok_or_else(|| "extract 需要 --input".to_string())?;
    let output =
        option(args, &["--output", "-o"])?.ok_or_else(|| "extract 需要 --output".to_string())?;
    let report = extract_workspace(&ExtractOptions {
        input: PathBuf::from(trim_drag_path(&input)),
        output: PathBuf::from(trim_drag_path(&output)),
        overwrite: has_flag(args, "--overwrite"),
    })?;
    println!("[extract] scanned_files={}", report.scanned_files);
    println!("[extract] json_files={}", report.json_files);
    println!("[extract] extracted_entries={}", report.extracted_entries);
    println!("[extract] dialogue_entries={}", report.dialogue_entries);
    println!("[extract] choice_entries={}", report.choice_entries);
    println!("[extract] name_entries={}", report.name_entries);
    println!("[extract] warnings={}", report.warnings);
    println!("[extract] output={}", report.output.display());
    Ok(())
}

fn run_inject_cli(args: &[String]) -> Result<(), BundleError> {
    reject_unknown_options(
        args,
        &[
            "--workspace",
            "--source",
            "--translations",
            "--output",
            "-o",
            "--overwrite",
            "--non-interactive",
        ],
    )?;
    let workspace =
        option(args, &["--workspace"])?.ok_or_else(|| "inject 需要 --workspace".to_string())?;
    let output =
        option(args, &["--output", "-o"])?.ok_or_else(|| "inject 需要 --output".to_string())?;
    let options = InjectOptions {
        workspace: PathBuf::from(trim_drag_path(&workspace)),
        source: option(args, &["--source"])?.map(|value| PathBuf::from(trim_drag_path(&value))),
        translations: option(args, &["--translations"])?
            .map(|value| PathBuf::from(trim_drag_path(&value))),
        output: PathBuf::from(trim_drag_path(&output)),
        overwrite: has_flag(args, "--overwrite"),
    };
    let report = inject_workspace(&options)?;
    println!("[inject] json_files={}", report.json_files);
    println!("[inject] json_entries={}", report.json_entries);
    println!("[inject] patched={}", report.patched);
    println!("[inject] unchanged={}", report.unchanged);
    println!("[inject] warnings={}", report.warnings);
    println!("[inject] output={}", report.output.display());
    Ok(())
}

fn read_line(prompt: &str) -> Option<String> {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).ok()? == 0 {
        return None;
    }
    Some(line.trim().to_string())
}

fn ask_path(label: &str, default: Option<&Path>) -> Option<PathBuf> {
    let suffix = default
        .map(|path| format!(" [{}]", path.display()))
        .unwrap_or_default();
    let value = read_line(&format!("{}{}: ", label, suffix))?;
    if value.is_empty() {
        return default.map(PathBuf::from);
    }
    Some(PathBuf::from(trim_drag_path(&value)))
}

fn ask_yes_no(prompt: &str) -> Option<bool> {
    loop {
        let value = read_line(&format!("{} [y/N]: ", prompt))?;
        match value.to_ascii_lowercase().as_str() {
            "y" | "yes" | "是" => return Some(true),
            "" | "n" | "no" | "否" => return Some(false),
            _ => println!("请输入 y 或 n。"),
        }
    }
}

fn output_conflict(path: &Path) -> Option<bool> {
    if !path.exists() {
        return Some(false);
    }
    loop {
        let value = read_line("输出已存在: [m] 修改路径 / [o] 覆盖 / [c] 取消: ")?;
        match value.to_ascii_lowercase().as_str() {
            "o" => return Some(true),
            "c" | "" => return None,
            "m" => return Some(false),
            _ => println!("请输入 m、o 或 c。"),
        }
    }
}

fn ask_output_path(label: &str, default: &Path) -> Option<(PathBuf, bool)> {
    let mut suggested = default.to_path_buf();
    loop {
        let output = ask_path(label, Some(&suggested))?;
        let existed = output.exists();
        match output_conflict(&output) {
            Some(true) => return Some((output, true)),
            Some(false) if existed => suggested = output,
            Some(false) => return Some((output, false)),
            None => return None,
        }
    }
}

fn interactive_extract(prefill: Option<PathBuf>) -> Result<(), BundleError> {
    let input = match ask_path("解包目录", prefill.as_deref()) {
        Some(path) => path,
        None => return Ok(()),
    };
    let bundle = Bundle::load(&input)?;
    let plan = build_plan(&bundle)?;
    let default_output = input
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            "{}_text_workspace",
            input
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("output")
        ));
    let (output, overwrite) = match ask_output_path("工作区输出目录", &default_output) {
        Some(value) => value,
        None => {
            println!("已取消，没有写入。\n");
            return Ok(());
        }
    };
    println!("\n识别为 AGSI SB2 解包目录: {}", input.display());
    println!(
        "将创建 {} 个 JSON，{} 个条目。",
        plan.report.json_files, plan.report.extracted_entries
    );
    println!("输出: {}", output.display());
    if !ask_yes_no("确认开始提取").unwrap_or(false) {
        println!("已取消，没有写入。\n");
        return Ok(());
    }
    let report = extract_workspace(&ExtractOptions {
        input,
        output,
        overwrite,
    })?;
    println!("提取完成: {}\n", report.output.display());
    Ok(())
}

fn interactive_inject(prefill: Option<PathBuf>) -> Result<(), BundleError> {
    let workspace = match ask_path("工作区目录", prefill.as_deref()) {
        Some(path) => path,
        None => return Ok(()),
    };
    let manifest = read_workspace(&workspace)?;
    let default_source = resolve_workspace_role(&workspace, &manifest.roles.source_root)?;
    let source = match ask_path("源快照目录", Some(&default_source)) {
        Some(path) => path,
        None => return Ok(()),
    };
    let default_translations =
        resolve_workspace_role(&workspace, &manifest.roles.translation_root)?;
    let translations = match ask_path("翻译 JSON 目录", Some(&default_translations)) {
        Some(path) => path,
        None => return Ok(()),
    };
    let default_output = workspace
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            "{}_injected",
            workspace
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("output")
        ));
    let (output, overwrite) = match ask_output_path("待封包目录", &default_output) {
        Some(value) => value,
        None => {
            println!("已取消，没有写入。\n");
            return Ok(());
        }
    };
    let options = InjectOptions {
        workspace,
        source: Some(source),
        translations: Some(translations),
        output,
        overwrite,
    };
    let preview = preview_inject(&options)?;
    println!(
        "\n工作区 JSON: {} 个文件，{} 个条目，待修改 CSTR: {}。",
        preview.json_files, preview.json_entries, preview.patched
    );
    println!("输出: {}", options.output.display());
    if !ask_yes_no("确认应用翻译并生成待封包目录").unwrap_or(false) {
        println!("已取消，没有写入。\n");
        return Ok(());
    }
    let report = inject_workspace(&options)?;
    println!("待封包目录已生成: {}\n", report.output.display());
    Ok(())
}

fn infer_prefill(path: &Path) -> Option<&'static str> {
    let is_workspace = path.join("workspace.json").is_file();
    let is_dump = path.join("manifest.json").is_file()
        && path.join("CODE.bin").is_file()
        && path.join("CSTR.bin").is_file();
    match (is_workspace, is_dump) {
        (true, false) => Some("inject"),
        (false, true) => Some("extract"),
        _ => None,
    }
}

fn interactive(prefill: Option<PathBuf>) -> Result<(), BundleError> {
    let mut menu_prefill = None;
    if let Some(path) = prefill {
        match infer_prefill(&path) {
            Some("extract") => {
                if let Err(error) = interactive_extract(Some(path)) {
                    println!("操作失败，可返回菜单修改: {}\n", error);
                }
            }
            Some("inject") => {
                if let Err(error) = interactive_inject(Some(path)) {
                    println!("操作失败，可返回菜单修改: {}\n", error);
                }
            }
            _ => {
                println!("无法唯一识别路径角色，请选择操作。");
                menu_prefill = Some(path);
            }
        }
    }
    loop {
        println!(
            "AGSI Sinfonia 文本工具\n1. 提取到受管工作区\n2. 应用翻译并生成待封包目录\n0. 退出"
        );
        let choice = match read_line("选择: ") {
            Some(value) => value,
            None => return Ok(()),
        };
        match choice.as_str() {
            "1" => {
                if let Err(error) = interactive_extract(menu_prefill.take()) {
                    println!("操作失败，可返回菜单修改: {}\n", error);
                }
            }
            "2" => {
                if let Err(error) = interactive_inject(menu_prefill.take()) {
                    println!("操作失败，可返回菜单修改: {}\n", error);
                }
            }
            "0" | "" => return Ok(()),
            _ => println!("请输入 1、2 或 0。\n"),
        }
    }
}

pub fn run(args: Vec<String>) -> Result<(), BundleError> {
    if args.is_empty() {
        return interactive(None);
    }
    if args[0] == "--help" || args[0] == "-h" {
        help();
        return Ok(());
    }
    match args[0].as_str() {
        "extract" => run_extract_cli(&args[1..]),
        "inject" => run_inject_cli(&args[1..]),
        _ => interactive(Some(PathBuf::from(trim_drag_path(&args[0])))),
    }
}
