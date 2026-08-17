use aitsuno_fdi_pack::{
    directory_contains_fdi, directory_contains_workspace, is_supported_fdi, is_workspace_manifest,
    pack_image, pack_workspace_tree, preview_workspace_pack,
};
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"fdi_pack 0.2.0

经结构校验的 PC-98 FDI/FAT12 递归重封盘工具。

用法:
  fdi_pack.exe pack --source <FDI_OR_DIR> --unpacked <UNPACKED_ROOT> --output <FDI_OR_DIR> [--overwrite]
  fdi_pack.exe pack --source <ORIGINAL.FDI> --workspace <WORKSPACE_JSON> --files <MEMBER_DIR> --output <NEW.FDI> [--overwrite]
  fdi_pack.exe [PATH ...]
  fdi_pack.exe

递归模式:
  --source <FILE_OR_DIR>    一张原始 FDI，或递归包含原始 FDI 的目录。
  --unpacked <DIR>         解包/注入后的完整根目录；递归按内容发现工作区清单。
  --output <FILE_OR_DIR>   source 为文件时必须是输出文件；source 为目录时必须是输出根目录。

递归模式按 SHA-256 匹配原盘与清单记录，不依赖 FDI 文件名。成员树由每份清单的
role_paths.unpacked_root 和 archives[].output_dir 定位；批量输出由清单相对位置和
archives[].source_file 稳定映射。全部盘完成预检和重建后才开始写输出。

兼容单盘模式:
  -s, --source <FILE>       原始 FDI 模板；必须由清单中的 SHA-256 唯一识别。
  -w, --workspace <FILE>    fdi_unpack 生成的 UTF-8 工作区清单；文件名任意。
  -f, --files <DIR>         对应磁盘的完整成员目录；名称任意。
  -o, --output <PATH>       显式输出文件或批量输出根目录。
      --unpacked <DIR>      选择递归模式；不能与 --workspace/--files 同时使用。
      --overwrite           允许事务式替换已存在的普通输出文件。
  -h, --help                显示帮助。

模式:
  完整的 pack 命令为一次性非交互操作。
  无参数或仅传入路径时进入持续交互会话；路径只用于可编辑预填，确认前不会写文件。

限制:
  支持现有成员内容变长、变短或等长修改；不新增、删除或重命名成员路径。
  空间不足、清单/源盘/成员哈希不匹配、额外或缺失成员时在写输出前拒绝。
"#;

enum Invocation {
    Help,
    ExplicitRecursive {
        source: PathBuf,
        unpacked: PathBuf,
        output: PathBuf,
        overwrite: bool,
    },
    ExplicitLegacy {
        source: PathBuf,
        workspace: PathBuf,
        files: PathBuf,
        output: PathBuf,
        overwrite: bool,
    },
    Interactive {
        prefill: Vec<PathBuf>,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("错误: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_invocation(env::args_os().skip(1).collect())? {
        Invocation::Help => {
            print!("{HELP}");
            Ok(())
        }
        Invocation::ExplicitRecursive {
            source,
            unpacked,
            output,
            overwrite,
        } => run_recursive_pack(&source, &unpacked, &output, overwrite),
        Invocation::ExplicitLegacy {
            source,
            workspace,
            files,
            output,
            overwrite,
        } => run_single_pack(&source, &workspace, &files, &output, overwrite),
        Invocation::Interactive { prefill } => interactive_session(prefill),
    }
}

fn parse_invocation(args: Vec<OsString>) -> Result<Invocation, String> {
    if args.is_empty() {
        return Ok(Invocation::Interactive {
            prefill: Vec::new(),
        });
    }
    if args.len() == 1 && matches!(args[0].to_str(), Some("-h" | "--help")) {
        return Ok(Invocation::Help);
    }
    if args[0] != "pack" {
        if args.iter().any(|arg| {
            arg.to_str()
                .map(|text| text.starts_with('-'))
                .unwrap_or(false)
        }) {
            return Err("未知参数；使用 --help 查看用法".to_string());
        }
        return Ok(Invocation::Interactive {
            prefill: args.into_iter().map(PathBuf::from).collect(),
        });
    }

    let mut source = None;
    let mut unpacked = None;
    let mut workspace = None;
    let mut files = None;
    let mut output = None;
    let mut overwrite = false;
    let mut index = 1usize;
    while index < args.len() {
        let argument = args[index]
            .to_str()
            .ok_or_else(|| "命令选项无法表示为 Unicode".to_string())?;
        match argument {
            "-h" | "--help" => return Ok(Invocation::Help),
            "-s" | "--source" => {
                index += 1;
                set_once(&mut source, args.get(index), argument)?;
            }
            "--unpacked" => {
                index += 1;
                set_once(&mut unpacked, args.get(index), argument)?;
            }
            "-w" | "--workspace" => {
                index += 1;
                set_once(&mut workspace, args.get(index), argument)?;
            }
            "-f" | "--files" => {
                index += 1;
                set_once(&mut files, args.get(index), argument)?;
            }
            "-o" | "--output" => {
                index += 1;
                set_once(&mut output, args.get(index), argument)?;
            }
            "--overwrite" => overwrite = true,
            value => return Err(format!("未知参数或缺少选项名: {value}")),
        }
        index += 1;
    }

    let source = source.ok_or_else(|| "pack 缺少 --source".to_string())?;
    let output = output.ok_or_else(|| "pack 缺少 --output".to_string())?;
    if let Some(unpacked) = unpacked {
        if workspace.is_some() || files.is_some() {
            return Err(
                "递归模式 --unpacked 不能与兼容单盘参数 --workspace/--files 同时使用".to_string(),
            );
        }
        return Ok(Invocation::ExplicitRecursive {
            source,
            unpacked,
            output,
            overwrite,
        });
    }
    Ok(Invocation::ExplicitLegacy {
        source,
        workspace: workspace
            .ok_or_else(|| "pack 缺少 --unpacked；兼容单盘模式则需要 --workspace".to_string())?,
        files: files.ok_or_else(|| "兼容单盘模式缺少 --files".to_string())?,
        output,
        overwrite,
    })
}

fn set_once(
    target: &mut Option<PathBuf>,
    value: Option<&OsString>,
    option: &str,
) -> Result<(), String> {
    let value = value.ok_or_else(|| format!("{option} 缺少路径"))?;
    if target.replace(PathBuf::from(value)).is_some() {
        return Err(format!("{option} 只能指定一次"));
    }
    Ok(())
}

fn run_recursive_pack(
    source: &Path,
    unpacked: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(), String> {
    let report = pack_workspace_tree(source, unpacked, output, overwrite)?;
    for item in &report.outputs {
        println!(
            "[pack:image] source_archive={} packed_files={} modified_files={} reallocated_files={} output_bytes={} output={}",
            item.source_archive,
            item.packed_files,
            item.modified_files,
            item.reallocated_files,
            item.output_bytes,
            item.output.display()
        );
    }
    println!("[pack] images={}", report.images);
    println!("[pack] packed_files={}", report.packed_files);
    println!("[pack] modified_files={}", report.modified_files);
    println!("[pack] reallocated_files={}", report.reallocated_files);
    println!(
        "[pack] skipped_source_images={}",
        report.skipped_source_images
    );
    println!("[pack] output_bytes={}", report.output_bytes);
    println!("[pack] output={}", output.display());
    Ok(())
}

fn run_single_pack(
    source: &Path,
    workspace: &Path,
    files: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(), String> {
    let report = pack_image(source, workspace, files, output, overwrite)?;
    println!("[pack] packed_files={}", report.packed_files);
    println!("[pack] modified_files={}", report.modified_files);
    println!("[pack] reallocated_files={}", report.reallocated_files);
    println!("[pack] free_clusters={}", report.free_clusters);
    println!("[pack] output_bytes={}", report.output_bytes);
    println!("[pack] source_archive={}", report.source_archive);
    println!("[pack] output={}", report.output.display());
    Ok(())
}

#[derive(Default)]
struct RecursiveForm {
    source: Option<PathBuf>,
    unpacked: Option<PathBuf>,
    output: Option<PathBuf>,
    overwrite: bool,
}

#[derive(Default)]
struct LegacyForm {
    source: Option<PathBuf>,
    workspace: Option<PathBuf>,
    files: Option<PathBuf>,
    output: Option<PathBuf>,
    overwrite: bool,
}

fn interactive_session(prefill: Vec<PathBuf>) -> Result<(), String> {
    if !prefill.is_empty() {
        if prefill
            .iter()
            .any(|path| path.is_file() && is_workspace_manifest(path))
        {
            let mut form = classify_legacy_prefill(&prefill);
            if let Err(error) = interactive_single_pack(&mut form, &prefill) {
                eprintln!("操作未完成: {error}");
            }
        } else {
            let mut form = classify_recursive_prefill(&prefill);
            if let Err(error) = interactive_recursive_pack(&mut form, &prefill) {
                eprintln!("操作未完成: {error}");
            }
        }
    }
    loop {
        println!();
        println!("FDI 重封盘工具");
        println!("1. 递归重封（单盘或批量）");
        println!("2. 兼容单盘重封（显式清单和成员目录）");
        println!("0. 退出");
        match prompt("请选择")?.trim() {
            "0" => return Ok(()),
            "1" => {
                let mut form = RecursiveForm::default();
                if let Err(error) = interactive_recursive_pack(&mut form, &[]) {
                    eprintln!("操作未完成: {error}");
                }
            }
            "2" => {
                let mut form = LegacyForm::default();
                if let Err(error) = interactive_single_pack(&mut form, &[]) {
                    eprintln!("操作未完成: {error}");
                }
            }
            _ => println!("无效选择，请重试。"),
        }
    }
}

fn classify_recursive_prefill(paths: &[PathBuf]) -> RecursiveForm {
    let mut form = RecursiveForm::default();
    for path in paths {
        if path.is_file() && is_supported_fdi(path) && form.source.is_none() {
            form.source = Some(path.clone());
        } else if path.is_dir() && directory_contains_workspace(path) && form.unpacked.is_none() {
            form.unpacked = Some(path.clone());
        } else if path.is_dir() && directory_contains_fdi(path) && form.source.is_none() {
            form.source = Some(path.clone());
        } else if form.output.is_none() {
            form.output = Some(path.clone());
        }
    }
    form
}

fn classify_legacy_prefill(paths: &[PathBuf]) -> LegacyForm {
    let mut form = LegacyForm::default();
    for path in paths {
        if path.is_file() && is_workspace_manifest(path) && form.workspace.is_none() {
            form.workspace = Some(path.clone());
        } else if path.is_file() && is_supported_fdi(path) && form.source.is_none() {
            form.source = Some(path.clone());
        } else if path.is_dir() && form.files.is_none() {
            form.files = Some(path.clone());
        }
    }
    form
}

fn interactive_recursive_pack(
    form: &mut RecursiveForm,
    candidates: &[PathBuf],
) -> Result<(), String> {
    loop {
        print_candidates(candidates);
        form.source = Some(prompt_path("原始 FDI 文件或目录", form.source.as_deref())?);
        form.unpacked = Some(prompt_path(
            "解包/注入后的完整根目录",
            form.unpacked.as_deref(),
        )?);
        if form.output.is_none() {
            form.output = form.source.as_deref().map(default_recursive_output);
        }
        form.output = Some(prompt_path(
            "输出 FDI 文件或输出根目录",
            form.output.as_deref(),
        )?);
        form.overwrite = prompt_overwrite(form.overwrite)?;

        let source = form.source.as_ref().expect("source was prompted");
        let unpacked = form.unpacked.as_ref().expect("unpacked was prompted");
        let output = form.output.as_ref().expect("output was prompted");
        let plan = match preview_workspace_pack(source, unpacked, output, form.overwrite) {
            Ok(plan) => plan,
            Err(error) => {
                eprintln!("预检失败: {error}");
                if prompt("输入 m 修改，输入 0 取消")?
                    .trim()
                    .eq_ignore_ascii_case("m")
                {
                    continue;
                }
                return Ok(());
            }
        };

        println!();
        println!("写入前确认（按清单和 SHA-256 识别）:");
        println!("  source={}", source.display());
        println!("  unpacked={}", unpacked.display());
        println!("  output={}", output.display());
        println!("  overwrite={}", form.overwrite);
        println!("  images={}", plan.mappings.len());
        println!("  skipped_source_images={}", plan.skipped_source_images);
        for mapping in &plan.mappings {
            println!(
                "  {}: {} + {} -> {}",
                mapping.archive_source_file,
                mapping.source.display(),
                mapping.files_root.display(),
                mapping.output.display()
            );
        }
        match prompt("输入 y 开始，输入 m 修改，输入 0 取消")?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "y" | "yes" => match run_recursive_pack(source, unpacked, output, form.overwrite) {
                Ok(()) => {
                    println!("操作完成，返回主菜单。");
                    return Ok(());
                }
                Err(error) => {
                    eprintln!("操作失败: {error}");
                    continue;
                }
            },
            "m" => continue,
            "0" | "" => return Ok(()),
            _ => println!("未确认写入，返回修改。"),
        }
    }
}

fn interactive_single_pack(form: &mut LegacyForm, candidates: &[PathBuf]) -> Result<(), String> {
    loop {
        print_candidates(candidates);
        form.source = Some(prompt_path("原始 FDI", form.source.as_deref())?);
        form.workspace = Some(prompt_path("工作区清单", form.workspace.as_deref())?);
        form.files = Some(prompt_path("修改后的完整成员目录", form.files.as_deref())?);
        if form.output.is_none() {
            form.output = form.source.as_deref().map(default_recursive_output);
        }
        form.output = Some(prompt_path("新 FDI 输出", form.output.as_deref())?);
        form.overwrite = prompt_overwrite(form.overwrite)?;

        println!();
        println!("写入前确认:");
        println!("  source={}", form.source.as_ref().unwrap().display());
        println!("  workspace={}", form.workspace.as_ref().unwrap().display());
        println!("  files={}", form.files.as_ref().unwrap().display());
        println!("  output={}", form.output.as_ref().unwrap().display());
        println!("  overwrite={}", form.overwrite);
        match prompt("输入 y 开始，输入 m 修改，输入 0 取消")?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "y" | "yes" => match run_single_pack(
                form.source.as_ref().unwrap(),
                form.workspace.as_ref().unwrap(),
                form.files.as_ref().unwrap(),
                form.output.as_ref().unwrap(),
                form.overwrite,
            ) {
                Ok(()) => {
                    println!("操作完成，返回主菜单。");
                    return Ok(());
                }
                Err(error) => {
                    eprintln!("操作失败: {error}");
                    continue;
                }
            },
            "m" => continue,
            "0" | "" => return Ok(()),
            _ => println!("未确认写入，返回修改。"),
        }
    }
}

fn default_recursive_output(source: &Path) -> PathBuf {
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    if source.is_dir() {
        let name = source
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("fdi");
        parent.join(format!("{name}_repacked"))
    } else {
        let stem = source
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("disk");
        parent.join(format!("{stem}_repacked.fdi"))
    }
}

fn print_candidates(candidates: &[PathBuf]) {
    if !candidates.is_empty() {
        println!();
        println!("路径预填候选（按内容识别，均可修改）:");
        for path in candidates {
            println!("  {}", path.display());
        }
    }
}

fn prompt_overwrite(current: bool) -> Result<bool, String> {
    Ok(matches!(
        prompt_with_default(
            "输出已存在时是否允许覆盖 [y/N]",
            if current { Some("y") } else { None },
        )?
        .trim()
        .to_ascii_lowercase()
        .as_str(),
        "y" | "yes"
    ))
}

fn prompt_path(label: &str, current: Option<&Path>) -> Result<PathBuf, String> {
    loop {
        let prompt_label = match current {
            Some(path) => format!("{label}（按 Enter 保留 {}，输入 0 取消）", path.display()),
            None => format!("{label}（输入 0 取消）"),
        };
        let value = prompt(&prompt_label)?;
        if value.trim() == "0" {
            return Err("用户取消".to_string());
        }
        if value.trim().is_empty() {
            if let Some(path) = current {
                return Ok(path.to_path_buf());
            }
            println!("此路径不能为空。");
            continue;
        }
        return Ok(PathBuf::from(strip_outer_quotes(value.trim())));
    }
}

fn prompt_with_default(label: &str, default: Option<&str>) -> Result<String, String> {
    let value = prompt(label)?;
    if value.trim().is_empty() {
        Ok(default.unwrap_or("").to_string())
    } else {
        Ok(value)
    }
}

fn strip_outer_quotes(value: &str) -> &str {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn prompt(label: &str) -> Result<String, String> {
    print!("{label}: ");
    io::stdout()
        .flush()
        .map_err(|e| format!("无法刷新终端输出: {e}"))?;
    let mut input = String::new();
    let bytes = io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("无法读取终端输入: {e}"))?;
    if bytes == 0 {
        return Ok("0".to_string());
    }
    Ok(input.trim_end_matches(['\r', '\n']).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_only_invocation_enters_interactive_mode() {
        let invocation = parse_invocation(vec![OsString::from("anything")]).expect("parse");
        match invocation {
            Invocation::Interactive { prefill } => {
                assert_eq!(prefill, vec![PathBuf::from("anything")]);
            }
            _ => panic!("expected interactive"),
        }
    }

    #[test]
    fn recursive_pack_requires_every_role() {
        let error = match parse_invocation(vec![
            OsString::from("pack"),
            OsString::from("--source"),
            OsString::from("source-dir"),
            OsString::from("--unpacked"),
            OsString::from("files-dir"),
        ]) {
            Ok(_) => panic!("missing output should fail"),
            Err(error) => error,
        };
        assert!(error.contains("--output"));
    }

    #[test]
    fn legacy_pack_requires_workspace_and_files() {
        let error = match parse_invocation(vec![
            OsString::from("pack"),
            OsString::from("--source"),
            OsString::from("source.fdi"),
            OsString::from("--output"),
            OsString::from("output.fdi"),
        ]) {
            Ok(_) => panic!("missing legacy roles should fail"),
            Err(error) => error,
        };
        assert!(error.contains("--unpacked"));
    }

    #[test]
    fn recursive_and_legacy_roles_are_mutually_exclusive() {
        let error = match parse_invocation(vec![
            OsString::from("pack"),
            OsString::from("--source"),
            OsString::from("source.fdi"),
            OsString::from("--unpacked"),
            OsString::from("tree"),
            OsString::from("--workspace"),
            OsString::from("workspace.json"),
            OsString::from("--output"),
            OsString::from("output.fdi"),
        ]) {
            Ok(_) => panic!("mixed roles should fail"),
            Err(error) => error,
        };
        assert!(error.contains("不能与"));
    }
}
