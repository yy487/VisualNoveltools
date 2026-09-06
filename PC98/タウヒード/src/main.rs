use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tauhido_nfd_unpacker::{
    localize::{extract_localization, pack_localization},
    preview_mappings, unpack_batch,
};

const HELP: &str = r#"tauhido_nfd_unpacker 0.1.0

Tauhido NFD R0 / N88 Disk BASIC 2HD 解包、文本本地化与重建工具。

用法:
  tauhido_nfd_unpacker.exe unpack --input <DISK_OR_DIR> [--input <...> ...] --output <DIR> [--overwrite]
  tauhido_nfd_unpacker.exe extract-localization --input <DISK_OR_DIR> [--input <...> ...] --output <WORKSPACE> [--overwrite]
  tauhido_nfd_unpacker.exe pack-localization --input <DISK_OR_DIR> [--input <...> ...] --workspace <WORKSPACE> --output <DIR> [--overwrite]
  tauhido_nfd_unpacker.exe <DISK_OR_DIR> [DISK_OR_DIR ...]
  tauhido_nfd_unpacker.exe

模式:
  完整命令为一次性非交互操作。
  无参数或仅传路径时进入交互模式；路径只用于预填，确认前不会写文件。

参数:
  -i, --input <PATH>  添加 NFD 文件或包含 NFD 的目录；可重复。
  -o, --output <DIR>  输出工作区。每张盘写入独立子目录。
  -w, --workspace <DIR>  pack-localization 使用的翻译工作区。
      --overwrite     只覆盖空目录或本工具生成的有效工作区。
  -h, --help          显示帮助。

输入识别:
  目录扫描只检查其直属普通文件，并根据 T98FDDIMAGE 签名识别，不依赖扩展名。

输出:
  extract-localization 提取 DISK-A、DISK-B、AG00 的 UTF-8 JSON。
  使用 --overwrite 刷新已有本地化工作区时，会按源文件、偏移和 scr_msg 保留匹配译文。
  pack-localization 注回文本、重建三张 NFD，并同步生成全槽位重绘的 font.tmp。
  所有命令都不会修改输入镜像。
"#;

enum Invocation {
    Help,
    Unpack {
        inputs: Vec<PathBuf>,
        output: PathBuf,
        overwrite: bool,
    },
    ExtractLocalization {
        inputs: Vec<PathBuf>,
        output: PathBuf,
        overwrite: bool,
    },
    PackLocalization {
        inputs: Vec<PathBuf>,
        workspace: PathBuf,
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
        Invocation::Unpack {
            inputs,
            output,
            overwrite,
        } => run_unpack(&inputs, &output, overwrite),
        Invocation::ExtractLocalization {
            inputs,
            output,
            overwrite,
        } => run_extract_localization(&inputs, &output, overwrite),
        Invocation::PackLocalization {
            inputs,
            workspace,
            output,
            overwrite,
        } => run_pack_localization(&inputs, &workspace, &output, overwrite),
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
    let command = args[0].to_str();
    if !matches!(
        command,
        Some("unpack" | "extract-localization" | "pack-localization")
    ) {
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

    let mut inputs = Vec::new();
    let mut output = None;
    let mut workspace = None;
    let mut overwrite = false;
    let mut index = 1usize;
    while index < args.len() {
        let argument = args[index]
            .to_str()
            .ok_or_else(|| "命令选项无法表示为 Unicode".to_string())?;
        match argument {
            "-h" | "--help" => return Ok(Invocation::Help),
            "-i" | "--input" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{argument} 缺少路径"))?;
                inputs.push(PathBuf::from(value));
            }
            "-o" | "--output" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{argument} 缺少路径"))?;
                if output.replace(PathBuf::from(value)).is_some() {
                    return Err("--output 只能指定一次".to_string());
                }
            }
            "-w" | "--workspace" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| format!("{argument} 缺少路径"))?;
                if workspace.replace(PathBuf::from(value)).is_some() {
                    return Err("--workspace 只能指定一次".to_string());
                }
            }
            "--overwrite" => overwrite = true,
            value if value.starts_with('-') => return Err(format!("未知参数: {value}")),
            _ => inputs.push(PathBuf::from(&args[index])),
        }
        index += 1;
    }
    let command = command.expect("validated command");
    if inputs.is_empty() {
        return Err(format!("{command} 至少需要一个 --input"));
    }
    let output = output.ok_or_else(|| format!("{command} 缺少 --output"))?;
    match command {
        "unpack" => {
            if workspace.is_some() {
                return Err("unpack 不接受 --workspace".to_string());
            }
            Ok(Invocation::Unpack {
                inputs,
                output,
                overwrite,
            })
        }
        "extract-localization" => {
            if workspace.is_some() {
                return Err("extract-localization 不接受 --workspace".to_string());
            }
            Ok(Invocation::ExtractLocalization {
                inputs,
                output,
                overwrite,
            })
        }
        "pack-localization" => Ok(Invocation::PackLocalization {
            inputs,
            workspace: workspace.ok_or_else(|| "pack-localization 缺少 --workspace".to_string())?,
            output,
            overwrite,
        }),
        _ => unreachable!(),
    }
}

fn run_unpack(inputs: &[PathBuf], output: &Path, overwrite: bool) -> Result<(), String> {
    let report = unpack_batch(inputs, output, overwrite)?;
    println!("[unpack] images={}", report.images);
    println!("[unpack] extracted_files={}", report.extracted_files);
    println!("[unpack] extracted_bytes={}", report.extracted_bytes);
    for warning in &report.warnings {
        eprintln!("[warning] {warning}");
    }
    println!("[unpack] output={}", report.output_root.display());
    Ok(())
}

fn run_extract_localization(
    inputs: &[PathBuf],
    output: &Path,
    overwrite: bool,
) -> Result<(), String> {
    let report = extract_localization(inputs, output, overwrite)?;
    println!("[extract-localization] images={}", report.images);
    println!(
        "[extract-localization] extracted_files={}",
        report.extracted_files
    );
    println!("[extract-localization] documents={}", report.documents);
    println!("[extract-localization] entries={}", report.entries);
    println!(
        "[extract-localization] preserved_entries={}",
        report.preserved_entries
    );
    println!(
        "[extract-localization] added_entries={}",
        report.added_entries
    );
    println!(
        "[extract-localization] dropped_entries={}",
        report.dropped_entries
    );
    println!(
        "[extract-localization] output={}",
        report.output_root.display()
    );
    Ok(())
}

fn run_pack_localization(
    inputs: &[PathBuf],
    workspace: &Path,
    output: &Path,
    overwrite: bool,
) -> Result<(), String> {
    let report = pack_localization(inputs, workspace, output, overwrite)?;
    println!("[pack-localization] images={}", report.images);
    println!("[pack-localization] documents={}", report.documents);
    println!("[pack-localization] entries={}", report.entries);
    println!(
        "[pack-localization] changed_entries={}",
        report.changed_entries
    );
    println!("[pack-localization] redrawn_slots={}", report.redrawn_slots);
    println!(
        "[pack-localization] output={}",
        report.output_root.display()
    );
    Ok(())
}

fn interactive_session(prefill: Vec<PathBuf>) -> Result<(), String> {
    let mut prefill = Some(prefill);
    loop {
        println!();
        println!("Tauhido NFD 本地化工具");
        println!("1. 仅解包 NFD");
        println!("2. 提取文本本地化工作区");
        println!("3. 注回文本、重建 NFD 和 font.tmp");
        println!("0. 退出");
        let choice = match prompt("请选择") {
            Ok(value) => value,
            Err(error) if error == "__EOF__" => return Ok(()),
            Err(error) => return Err(error),
        };
        match choice.trim() {
            "0" => return Ok(()),
            "1" => {
                let mut inputs = prefill.take().unwrap_or_default();
                if let Err(error) = interactive_unpack(&mut inputs) {
                    if error == "__EOF__" {
                        return Ok(());
                    }
                    eprintln!("操作未完成: {error}");
                }
            }
            "2" => {
                let mut inputs = prefill.take().unwrap_or_default();
                if let Err(error) = interactive_extract_localization(&mut inputs) {
                    if error == "__EOF__" {
                        return Ok(());
                    }
                    eprintln!("操作未完成: {error}");
                }
            }
            "3" => {
                let mut inputs = prefill.take().unwrap_or_default();
                if let Err(error) = interactive_pack_localization(&mut inputs) {
                    if error == "__EOF__" {
                        return Ok(());
                    }
                    eprintln!("操作未完成: {error}");
                }
            }
            _ => println!("无效选择，请重试。"),
        }
    }
}

fn interactive_extract_localization(inputs: &mut Vec<PathBuf>) -> Result<(), String> {
    loop {
        println!();
        if inputs.is_empty() {
            println!("请输入三张原始 NFD 或其所在目录，每行一个，空行结束；输入 0 返回。");
            *inputs = read_paths()?;
            if inputs.is_empty() {
                return Ok(());
            }
        } else {
            println!("当前原始 NFD 输入:");
            for (index, path) in inputs.iter().enumerate() {
                println!("  {}. {}", index + 1, path.display());
            }
            let action = prompt("按 Enter 保留，输入 r 重新选择，输入 0 返回")?;
            match action.trim().to_ascii_lowercase().as_str() {
                "" => {}
                "r" => {
                    inputs.clear();
                    continue;
                }
                "0" => return Ok(()),
                _ => {
                    println!("无效选择。");
                    continue;
                }
            }
        }

        let mappings = preview_mappings(inputs)?;
        println!("检测到的镜像:");
        for (source, member_dir) in &mappings {
            println!("  {} -> extract/{}", source.display(), member_dir);
        }
        let default_output = suggested_sibling(inputs, "tauhido_localization_workspace");
        let output_text = prompt(&format!(
            "本地化工作区（按 Enter 使用建议值 {}，输入 0 返回）",
            default_output.display()
        ))?;
        if output_text.trim() == "0" {
            return Ok(());
        }
        let output = if output_text.trim().is_empty() {
            default_output
        } else {
            PathBuf::from(strip_outer_quotes(output_text.trim()))
        };
        let overwrite = matches!(
            prompt("若工作区已存在，是否刷新并保留匹配译文？[y/N]")?
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "y" | "yes"
        );
        println!("即将提取 DISK-A、DISK-B 和 AG00 到 {}", output.display());
        println!("NACT8S 不会进入本地化工作区。");
        if !matches!(
            prompt("确认写入？[y/N]")?
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "y" | "yes"
        ) {
            println!("已取消。未写入任何文件。");
            return Ok(());
        }
        run_extract_localization(inputs, &output, overwrite)?;
        return Ok(());
    }
}

fn interactive_pack_localization(inputs: &mut Vec<PathBuf>) -> Result<(), String> {
    loop {
        println!();
        if inputs.is_empty() {
            println!("请输入三张原始 NFD 或其所在目录，每行一个，空行结束；输入 0 返回。");
            *inputs = read_paths()?;
            if inputs.is_empty() {
                return Ok(());
            }
        } else {
            println!("当前原始 NFD 输入:");
            for (index, path) in inputs.iter().enumerate() {
                println!("  {}. {}", index + 1, path.display());
            }
            let action = prompt("按 Enter 保留，输入 r 重新选择，输入 0 返回")?;
            match action.trim().to_ascii_lowercase().as_str() {
                "" => {}
                "r" => {
                    inputs.clear();
                    continue;
                }
                "0" => return Ok(()),
                _ => {
                    println!("无效选择。");
                    continue;
                }
            }
        }

        let mappings = preview_mappings(inputs)?;
        println!("检测到的原始镜像:");
        for (source, _) in &mappings {
            println!("  {}", source.display());
        }
        let default_workspace = suggested_sibling(inputs, "tauhido_localization_workspace");
        let workspace_text = prompt(&format!(
            "翻译工作区（按 Enter 使用建议值 {}，输入 0 返回）",
            default_workspace.display()
        ))?;
        if workspace_text.trim() == "0" {
            return Ok(());
        }
        let workspace = if workspace_text.trim().is_empty() {
            default_workspace
        } else {
            PathBuf::from(strip_outer_quotes(workspace_text.trim()))
        };
        let default_output = workspace.join("rebuild");
        let output_text = prompt(&format!(
            "重建输出目录（按 Enter 使用建议值 {}，输入 0 返回）",
            default_output.display()
        ))?;
        if output_text.trim() == "0" {
            return Ok(());
        }
        let output = if output_text.trim().is_empty() {
            default_output
        } else {
            PathBuf::from(strip_outer_quotes(output_text.trim()))
        };
        let overwrite = matches!(
            prompt("若输出已存在，是否覆盖本工具重建目录？[y/N]")?
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "y" | "yes"
        );
        println!("原始 NFD: {} 个输入", inputs.len());
        println!("翻译工作区: {}", workspace.display());
        println!("重建输出: {}", output.display());
        println!("输出将包含三张 NFD 和同步重绘的 font.tmp。");
        if !matches!(
            prompt("确认写入？[y/N]")?
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "y" | "yes"
        ) {
            println!("已取消。未写入任何文件。");
            return Ok(());
        }
        run_pack_localization(inputs, &workspace, &output, overwrite)?;
        return Ok(());
    }
}

fn suggested_sibling(inputs: &[PathBuf], name: &str) -> PathBuf {
    if inputs[0].is_dir() {
        inputs[0].join(name)
    } else {
        inputs[0]
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(name)
    }
}

fn interactive_unpack(inputs: &mut Vec<PathBuf>) -> Result<(), String> {
    loop {
        println!();
        if inputs.is_empty() {
            println!("请输入 NFD 文件或目录，每行一个，空行结束；输入 0 返回。");
            *inputs = read_paths()?;
            if inputs.is_empty() {
                return Ok(());
            }
        } else {
            println!("当前输入:");
            for (index, path) in inputs.iter().enumerate() {
                println!("  {}. {}", index + 1, path.display());
            }
            let action = prompt("按 Enter 保留，输入 r 重新选择，输入 0 返回")?;
            match action.trim().to_ascii_lowercase().as_str() {
                "" => {}
                "r" => {
                    inputs.clear();
                    continue;
                }
                "0" => return Ok(()),
                _ => {
                    println!("无效选择。");
                    continue;
                }
            }
        }

        let mappings = preview_mappings(inputs)?;
        println!("检测到的镜像:");
        for (source, member_dir) in &mappings {
            println!("  {} -> {}", source.display(), member_dir);
        }

        let default_output = if inputs[0].is_dir() {
            inputs[0].join("tauhido_nfd_unpacked")
        } else {
            inputs[0]
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("tauhido_nfd_unpacked")
        };
        let output_text = prompt(&format!(
            "输出目录（按 Enter 使用建议值 {}，输入 0 返回）",
            default_output.display()
        ))?;
        if output_text.trim() == "0" {
            return Ok(());
        }
        let output = if output_text.trim().is_empty() {
            default_output
        } else {
            PathBuf::from(strip_outer_quotes(output_text.trim()))
        };
        let overwrite = matches!(
            prompt("若输出已存在，是否覆盖本工具工作区？[y/N]")?
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "y" | "yes"
        );

        println!("即将解包 {} 张镜像到 {}", mappings.len(), output.display());
        if !matches!(
            prompt("确认写入？[y/N]")?
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "y" | "yes"
        ) {
            println!("已取消。未写入任何文件。");
            return Ok(());
        }
        run_unpack(inputs, &output, overwrite)?;
        return Ok(());
    }
}

fn read_paths() -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    loop {
        let line = prompt("路径")?;
        let value = line.trim();
        if value.is_empty() {
            break;
        }
        if value == "0" && paths.is_empty() {
            return Ok(Vec::new());
        }
        paths.push(PathBuf::from(strip_outer_quotes(value)));
    }
    Ok(paths)
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
        .map_err(|error| format!("刷新终端失败: {error}"))?;
    let mut input = String::new();
    let count = io::stdin()
        .read_line(&mut input)
        .map_err(|error| format!("读取输入失败: {error}"))?;
    if count == 0 {
        return Err("__EOF__".to_string());
    }
    Ok(input.trim_end_matches(['\r', '\n']).to_string())
}
