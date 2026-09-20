use genji_unpack::localization::{extract_localization, inject_localization};
use genji_unpack::{rebuild_fdi, unpack, Result};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn help() {
    println!(
        "源氏 FDI 资源与汉化工具\n\n\
  genji_unpack unpack <input.fdi> <output-dir> [--overwrite]\n\
  genji_unpack extract-text <input.fdi> <workspace-dir> [--overwrite]\n\
  genji_unpack inject-text <input.fdi> <workspace-dir> <output-dir> [--overwrite]\n\
  genji_unpack rebuild-fdi <input.fdi> <unpacked-dir> <output.fdi> [--overwrite]\n\
  genji_unpack --help\n\n\
双击或不带参数运行可使用交互菜单；genji_gui.exe 提供 Windows 图形界面。\n\
extract-text 输出 UTF-8 JSON；name 与 message 是可编辑字段。\n\
inject-text 同时输出重建后的 FDI、font.tmp 和 rebuild_manifest.json。"
    );
}

fn ask(label: &str, default: &str) -> Option<String> {
    print!("{label} [{default}]: ");
    io::stdout().flush().ok()?;
    let mut value = String::new();
    if io::stdin().read_line(&mut value).ok()? == 0 {
        return None;
    }
    let value = value.trim().trim_matches('"');
    Some(if value.is_empty() {
        default.to_owned()
    } else {
        value.to_owned()
    })
}

fn yes(label: &str, default: bool) -> Option<bool> {
    let default_text = if default { "y" } else { "n" };
    ask(label, default_text).map(|value| value.eq_ignore_ascii_case("y"))
}

fn interactive(prefill: Option<String>) {
    let mut input = prefill.unwrap_or_default();
    loop {
        println!(
            "\n1. 解包原始资源\n2. 提取翻译 JSON\n3. 注入文本并重建 FDI/字库\n4. 从资源目录重建 FDI\n0. 退出"
        );
        let Some(choice) = ask("操作", "1") else {
            return;
        };
        if choice == "0" {
            return;
        }
        let Some(source) = ask("源 FDI", &input) else {
            return;
        };
        input = source.clone();
        let result = match choice.as_str() {
            "1" => {
                let suggested = PathBuf::from(&source).with_extension("unpacked");
                let Some(output) = ask("输出目录", &suggested.to_string_lossy()) else {
                    return;
                };
                let Some(overwrite) = yes("覆盖已有文件？y/n", false) else {
                    return;
                };
                unpack(Path::new(&source), Path::new(&output), overwrite).map(|report| {
                    format!(
                        "完成：{} 个盘内文件，{} 个游戏资源 -> {output}",
                        report.files.len(),
                        report.resource_count
                    )
                })
            }
            "2" => {
                let suggested = PathBuf::from(&source)
                    .with_file_name("localization")
                    .to_string_lossy()
                    .into_owned();
                let Some(output) = ask("翻译工作区", &suggested) else {
                    return;
                };
                let Some(overwrite) = yes("覆盖并保留已有译文？y/n", false) else {
                    return;
                };
                extract_localization(Path::new(&source), Path::new(&output), overwrite).map(
                    |report| {
                        format!(
                            "完成：{} 个文本条目，{} 个姓名槽，{} 个选项 -> {}",
                            report.entries,
                            report.names,
                            report.choices,
                            report.output.display()
                        )
                    },
                )
            }
            "3" => {
                let Some(workspace) = ask("翻译工作区", "localization") else {
                    return;
                };
                let Some(output) = ask("回包输出目录", "rebuilt") else {
                    return;
                };
                let Some(overwrite) = yes("覆盖已有受管输出？y/n", false) else {
                    return;
                };
                inject_localization(
                    Path::new(&source),
                    Path::new(&workspace),
                    Path::new(&output),
                    overwrite,
                )
                .map(|report| {
                    format!(
                        "完成：改动 {} 条，重建 {} 张文本表，字槽 {} 个 -> {}",
                        report.changed_entries,
                        report.changed_tables,
                        report.redrawn_slots,
                        report.output_fdi.display()
                    )
                })
            }
            "4" => {
                let Some(unpacked) = ask("解包目录", "unpacked") else {
                    return;
                };
                let Some(output) = ask("输出 FDI", "Genji_rebuilt.FDI") else {
                    return;
                };
                let Some(overwrite) = yes("覆盖已有文件？y/n", false) else {
                    return;
                };
                rebuild_fdi(
                    Path::new(&source),
                    Path::new(&unpacked),
                    Path::new(&output),
                    overwrite,
                )
                .map(|report| {
                    format!(
                        "完成：{} 个资源发生变化，G1.DAT {} 字节 -> {}",
                        report.changed_resources, report.g1_size, report.output
                    )
                })
            }
            _ => continue,
        };
        match result {
            Ok(message) => println!("{message}"),
            Err(error) => eprintln!("错误：{error}"),
        }
    }
}

fn overwrite_flag(args: &[String], expected_without_flag: usize) -> Result<bool> {
    match args.len() {
        length if length == expected_without_flag => Ok(false),
        length if length == expected_without_flag + 1 && args.last().unwrap() == "--overwrite" => {
            Ok(true)
        }
        _ => Err("invalid argument count or option".into()),
    }
}

fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|value| value == "-h" || value == "--help") {
        help();
        return Ok(());
    }
    if args.is_empty() || (args.len() == 1 && !args[0].starts_with('-')) {
        interactive(args.first().cloned());
        return Ok(());
    }
    match args[0].as_str() {
        "unpack" => {
            let overwrite = overwrite_flag(&args, 3).map_err(|_| {
                "Usage: genji_unpack unpack <input.fdi> <output-dir> [--overwrite]".to_string()
            })?;
            let report = unpack(Path::new(&args[1]), Path::new(&args[2]), overwrite)?;
            println!(
                "完成：{} 个盘内文件，{} 个游戏资源 -> {}",
                report.files.len(),
                report.resource_count,
                args[2]
            );
        }
        "extract-text" => {
            let overwrite = overwrite_flag(&args, 3).map_err(|_| {
                "Usage: genji_unpack extract-text <input.fdi> <workspace-dir> [--overwrite]"
                    .to_string()
            })?;
            let report = extract_localization(Path::new(&args[1]), Path::new(&args[2]), overwrite)?;
            println!(
                "完成：{} 条文本，{} 个姓名槽，{} 个选项，保留 {} 条已有译文 -> {}",
                report.entries,
                report.names,
                report.choices,
                report.preserved_translations,
                report.output.display()
            );
        }
        "inject-text" => {
            let overwrite = overwrite_flag(&args, 4).map_err(|_| {
                "Usage: genji_unpack inject-text <input.fdi> <workspace-dir> <output-dir> [--overwrite]"
                    .to_string()
            })?;
            let report = inject_localization(
                Path::new(&args[1]),
                Path::new(&args[2]),
                Path::new(&args[3]),
                overwrite,
            )?;
            println!(
                "完成：改动 {} 条，重建 {} 张文本表，字槽 {} 个 -> {}",
                report.changed_entries,
                report.changed_tables,
                report.redrawn_slots,
                report.output_fdi.display()
            );
        }
        "rebuild-fdi" => {
            let overwrite = overwrite_flag(&args, 4).map_err(|_| {
                "Usage: genji_unpack rebuild-fdi <input.fdi> <unpacked-dir> <output.fdi> [--overwrite]"
                    .to_string()
            })?;
            let report = rebuild_fdi(
                Path::new(&args[1]),
                Path::new(&args[2]),
                Path::new(&args[3]),
                overwrite,
            )?;
            println!(
                "完成：{} 个资源发生变化，G1.DAT {} 字节 -> {}",
                report.changed_resources, report.g1_size, report.output
            );
        }
        _ => return Err("unknown command; use --help".into()),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("错误：{error}");
        std::process::exit(1);
    }
}
