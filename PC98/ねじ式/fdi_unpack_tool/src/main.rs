use nejishiki_fdi_unpack::{preview_mappings, unpack_batch};
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const HELP: &str = r#"fdi_unpack 0.1.0

经结构校验的 PC-98 FDI/FAT12 批量解包器。

用法:
  fdi_unpack.exe unpack --input <DISK.FDI> [--input <DISK.FDI> ...] --output <DIR> [--overwrite]
  fdi_unpack.exe <DISK.FDI> [DISK.FDI ...]
  fdi_unpack.exe

模式:
  完整的 unpack 命令为一次性非交互操作。
  无参数或仅传入路径时进入交互模式；路径只用于预填，确认前不会写文件。

参数:
  -i, --input <FILE>   添加一个输入；可重复。也可在 unpack 后使用位置参数。
  -o, --output <DIR>   批量输出根目录。每张盘映射到以源文件名建议的子目录。
      --overwrite      仅覆盖空目录或本工具生成且含有效 workspace.json 的工作区。
  -h, --help           显示帮助。

输出:
  <DIR>/workspace.json 保存源盘哈希、FDI/BPB、目录项、簇链和文件哈希。
  已存在的输出默认拒绝；不会覆盖或修改输入 FDI。
"#;

enum Invocation {
    Help,
    Explicit {
        inputs: Vec<PathBuf>,
        output: PathBuf,
        overwrite: bool,
    },
    Interactive {
        prefill: Vec<PathBuf>,
    },
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("错误: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    match parse_invocation(env::args_os().skip(1).collect())? {
        Invocation::Help => {
            print!("{HELP}");
            Ok(())
        }
        Invocation::Explicit {
            inputs,
            output,
            overwrite,
        } => run_unpack(&inputs, &output, overwrite),
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
    if args[0] != "unpack" {
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
            "--overwrite" => overwrite = true,
            value if value.starts_with('-') => return Err(format!("未知参数: {value}")),
            _ => inputs.push(PathBuf::from(&args[index])),
        }
        index += 1;
    }
    if inputs.is_empty() {
        return Err("unpack 至少需要一个 --input".to_string());
    }
    let output = output.ok_or_else(|| "unpack 缺少 --output".to_string())?;
    Ok(Invocation::Explicit {
        inputs,
        output,
        overwrite,
    })
}

fn run_unpack(inputs: &[PathBuf], output: &Path, overwrite: bool) -> Result<(), String> {
    let report = unpack_batch(inputs, output, overwrite)?;
    println!("[unpack] images={}", report.images);
    println!("[unpack] extracted_files={}", report.extracted_files);
    println!(
        "[unpack] extracted_directories={}",
        report.extracted_directories
    );
    println!("[unpack] extracted_bytes={}", report.extracted_bytes);
    for warning in &report.warnings {
        eprintln!("[warning] {warning}");
    }
    println!("[unpack] output={}", report.output_root.display());
    Ok(())
}

fn interactive_session(mut prefill: Vec<PathBuf>) -> Result<(), String> {
    if !prefill.is_empty() {
        interactive_unpack(&mut prefill)?;
    }
    loop {
        println!();
        println!("FDI 解包器");
        println!("1. 解包 FDI");
        println!("0. 退出");
        match prompt("请选择")?.trim() {
            "0" => return Ok(()),
            "1" => {
                let mut inputs = Vec::new();
                if let Err(error) = interactive_unpack(&mut inputs) {
                    eprintln!("操作未完成: {error}");
                }
            }
            _ => println!("无效选择，请重试。"),
        }
    }
}

fn interactive_unpack(inputs: &mut Vec<PathBuf>) -> Result<(), String> {
    loop {
        println!();
        if inputs.is_empty() {
            println!("尚未选择输入 FDI。请输入路径，每行一个，空行结束；输入 0 返回。");
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
        let default_output = inputs
            .first()
            .and_then(|path| path.parent())
            .unwrap_or_else(|| Path::new("."))
            .join("fdi_unpacked");
        let output_text = prompt(&format!(
            "输出根目录（按 Enter 使用建议值 {}）",
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

        println!();
        println!("写入前确认:");
        for (input, subdir) in mappings {
            println!("  {} -> {}/{}", input.display(), output.display(), subdir);
        }
        println!("  overwrite={overwrite}");
        match prompt("输入 y 开始，输入 m 修改，输入 0 取消")?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "y" | "yes" => {
                match run_unpack(inputs, &output, overwrite) {
                    Ok(()) => println!("操作完成，返回主菜单。"),
                    Err(error) => eprintln!("操作失败: {error}"),
                }
                return Ok(());
            }
            "m" => continue,
            "0" | "" => return Ok(()),
            _ => println!("未确认写入，返回修改。"),
        }
    }
}

fn read_paths() -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    loop {
        let line = prompt("FDI 路径")?;
        let value = line.trim();
        if value.is_empty() {
            break;
        }
        if value == "0" && paths.is_empty() {
            break;
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
    fn path_only_invocation_is_interactive_prefill() {
        let invocation = parse_invocation(vec![OsString::from("disk.fdi")]).expect("parse");
        match invocation {
            Invocation::Interactive { prefill } => {
                assert_eq!(prefill, vec![PathBuf::from("disk.fdi")])
            }
            _ => panic!("expected interactive mode"),
        }
    }

    #[test]
    fn explicit_invocation_requires_output() {
        let error = match parse_invocation(vec![
            OsString::from("unpack"),
            OsString::from("--input"),
            OsString::from("disk.fdi"),
        ]) {
            Ok(_) => panic!("missing output should fail"),
            Err(error) => error,
        };
        assert!(error.contains("--output"));
    }
}
