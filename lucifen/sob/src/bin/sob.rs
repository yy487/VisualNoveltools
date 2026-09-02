use oretubar_tob_sob_tool::common::{
    collect_files, confirm, ensure_output, parse_args, prompt, EncodingChoice, Result,
};
use oretubar_tob_sob_tool::sob::{self, RepairIssue};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

struct RepairArgs {
    input: PathBuf,
    baseline: PathBuf,
    output: PathBuf,
    report: PathBuf,
    overwrite: bool,
}

#[derive(Serialize)]
struct RepairReport {
    changed_files: usize,
    changed_entries: usize,
    issues: Vec<RepairIssue>,
}

fn json_path(root: &Path, rel: &Path) -> PathBuf {
    root.join(rel).with_extension("json")
}

fn parse_repair_args(args: &[String]) -> Result<RepairArgs> {
    let mut input = None;
    let mut baseline = None;
    let mut output = None;
    let mut report = None;
    let mut encoding = None;
    let mut overwrite = false;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                input = args.get(i).map(PathBuf::from);
            }
            "--baseline" => {
                i += 1;
                baseline = args.get(i).map(PathBuf::from);
            }
            "--output" => {
                i += 1;
                output = args.get(i).map(PathBuf::from);
            }
            "--report" => {
                i += 1;
                report = args.get(i).map(PathBuf::from);
            }
            "--encoding" => {
                i += 1;
                encoding = Some(EncodingChoice::parse(
                    args.get(i).ok_or("missing --encoding value")?,
                )?);
            }
            "--overwrite" => overwrite = true,
            "--help" | "-h" => {
                return Err("usage: repair --input CHS_PATH --baseline ROW_PATH --output PATH --report PATH --encoding gbk [--overwrite]".into());
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
        i += 1;
    }
    if encoding != Some(EncodingChoice::Gbk) {
        return Err("repair requires --encoding gbk".into());
    }
    Ok(RepairArgs {
        input: input.ok_or("missing --input")?,
        baseline: baseline.ok_or("missing --baseline")?,
        output: output.ok_or("missing --output")?,
        report: report.ok_or("missing --report")?,
        overwrite,
    })
}

fn write_report(path: &Path, report: &RepairReport, overwrite: bool) -> Result<()> {
    if path.exists() && !overwrite {
        return Err(format!(
            "report exists: {} (use --overwrite)",
            path.display()
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| format!("write {}: {e}", path.display()))
}

fn merge_issues(issues: Vec<RepairIssue>) -> Vec<RepairIssue> {
    let mut merged = HashMap::<(String, Option<u64>, String), RepairIssue>::new();
    for issue in issues {
        let key = (issue.file.clone(), issue.offset, issue.reason.clone());
        if let Some(existing) = merged.get_mut(&key) {
            existing.indexes.extend(issue.indexes);
            if existing.context.is_empty() {
                existing.context = issue.context;
            }
        } else {
            merged.insert(key, issue);
        }
    }
    let mut issues: Vec<_> = merged.into_values().collect();
    for issue in &mut issues {
        issue.indexes.sort_unstable();
        issue.indexes.dedup();
    }
    issues.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then_with(|| left.offset.cmp(&right.offset))
    });
    issues
}

fn run_repair(args: RepairArgs) -> Result<()> {
    let files = collect_files(&args.input, "sob")?;
    let baseline_files = collect_files(&args.baseline, "sob")?;
    let mut baseline_by_rel = HashMap::new();
    for (path, rel) in baseline_files {
        baseline_by_rel.insert(rel, path);
    }
    if args.input.is_dir() {
        ensure_output(&args.output, args.overwrite)?;
    }
    let mut report = RepairReport {
        changed_files: 0,
        changed_entries: 0,
        issues: Vec::new(),
    };
    for (source, rel) in files {
        let baseline = if args.input.is_file() {
            args.baseline.clone()
        } else {
            match baseline_by_rel.get(&rel) {
                Some(path) => path.clone(),
                None => {
                    report.issues.push(RepairIssue {
                        file: rel.to_string_lossy().into_owned(),
                        indexes: Vec::new(),
                        offset: None,
                        reason: "matching ROW file is missing".into(),
                        source: None,
                        message: None,
                        context: Vec::new(),
                    });
                    let destination = args.output.join(&rel);
                    if destination.exists() && !args.overwrite {
                        return Err(format!(
                            "output exists: {} (use --overwrite)",
                            destination.display()
                        ));
                    }
                    if let Some(parent) = destination.parent() {
                        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    fs::copy(&source, &destination).map_err(|e| e.to_string())?;
                    continue;
                }
            }
        };
        let destination = if args.input.is_file() {
            args.output.clone()
        } else {
            args.output.join(&rel)
        };
        let summary = sob::repair_file(
            &source,
            &baseline,
            &destination,
            rel.to_string_lossy().as_ref(),
            args.overwrite,
        )?;
        if summary.changed > 0 {
            report.changed_files += 1;
            report.changed_entries += summary.changed;
        }
        report.issues.extend(summary.issues);
    }
    report.issues = merge_issues(report.issues);
    write_report(&args.report, &report, args.overwrite)?;
    println!(
        "repaired {} files, updated {} offsets, unresolved {} entries; report {}",
        report.changed_files,
        report.changed_entries,
        report.issues.len(),
        args.report.display()
    );
    Ok(())
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        return interactive(None);
    }
    if args.len() == 2
        && !matches!(
            args[1].as_str(),
            "extract" | "inject" | "repair" | "--help" | "-h"
        )
    {
        return interactive(Some(PathBuf::from(&args[1])));
    }
    if args.iter().any(|x| x == "--help" || x == "-h") {
        println!("oretubar-sob\n  extract --input PATH --output PATH --encoding sjis|gbk [--overwrite]\n  inject  --input PATH --translation PATH --output PATH --encoding sjis|gbk [--overwrite]\n  repair  --input CHS_PATH --baseline ROW_PATH --output PATH --report PATH --encoding gbk [--overwrite]");
        return Ok(());
    }
    if args.get(1).map(String::as_str) == Some("repair") {
        return run_repair(parse_repair_args(&args)?);
    }
    let (mode, input, output, encoding, overwrite) = parse_args(&args)?;
    let files = collect_files(&input, "sob")?;
    if mode == "extract" {
        if input.is_dir() {
            ensure_output(&output, overwrite)?;
        }
        let mut total = 0;
        for (source, rel) in files {
            let destination = if input.is_file() {
                output.clone()
            } else {
                output.join(&rel).with_extension("json")
            };
            total += sob::extract_file(
                &source,
                &destination,
                rel.to_string_lossy().as_ref(),
                encoding,
                overwrite,
            )?;
        }
        println!(
            "extracted {total} SOB entries from {} files",
            collect_files(&input, "sob")?.len()
        );
    } else if mode == "inject" {
        if input.is_dir() {
            ensure_output(&output, overwrite)?;
        }
        let translation = args
            .iter()
            .position(|x| x == "--translation")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
            .ok_or("inject requires --translation")?;
        let mut changed = 0;
        for (source, rel) in files {
            let trans = if translation.is_file() {
                translation.clone()
            } else {
                json_path(&translation, &rel)
            };
            if !trans.exists() {
                return Err(format!("missing translation {}", trans.display()));
            }
            let destination = if input.is_file() {
                output.clone()
            } else {
                output.join(&rel)
            };
            changed += sob::inject_file(&source, &trans, &destination, encoding, overwrite)?;
        }
        println!("updated {changed} SOB records");
    } else {
        return Err(format!("unknown mode '{mode}'"));
    }
    Ok(())
}

fn interactive(mut prefill: Option<PathBuf>) -> Result<()> {
    loop {
        println!("\nORETUBAR SOB tool\n  1) Extract\n  2) Inject\n  3) Repair offsets\n  4) Exit");
        let choice = prompt("Choice", None)?;
        match choice.trim().to_ascii_lowercase().as_str() {
            "1" | "extract" => match interactive_extract(prefill.take()) {
                Ok(()) => {}
                Err(error) if error == "end of input" => return Err(error),
                Err(error) => eprintln!("error: {error}"),
            },
            "2" | "inject" => match interactive_inject(prefill.take()) {
                Ok(()) => {}
                Err(error) if error == "end of input" => return Err(error),
                Err(error) => eprintln!("error: {error}"),
            },
            "3" | "repair" => match interactive_repair(prefill.take()) {
                Ok(()) => {}
                Err(error) if error == "end of input" => return Err(error),
                Err(error) => eprintln!("error: {error}"),
            },
            "4" | "exit" | "quit" | "q" => return Ok(()),
            "" => {}
            _ => eprintln!("Unknown choice."),
        }
    }
}

fn interactive_extract(prefill: Option<PathBuf>) -> Result<()> {
    let default_input = prefill.as_deref().and_then(Path::to_str);
    let input = PathBuf::from(prompt("Input SOB file/directory", default_input)?);
    let default_output = if input.is_file() {
        "translation.json"
    } else {
        "translation_json"
    };
    let output = PathBuf::from(prompt("JSON output file/directory", Some(default_output))?);
    let encoding = EncodingChoice::parse(&prompt("Source encoding (sjis/gbk)", None)?)?;
    let overwrite = if output.exists() {
        confirm("Output exists; overwrite it")?
    } else {
        false
    };
    if output.exists() && !overwrite {
        println!("Cancelled.");
        return Ok(());
    }
    println!(
        "Extract {} -> {} ({})",
        input.display(),
        output.display(),
        encoding.label()
    );
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    let files = collect_files(&input, "sob")?;
    if input.is_dir() {
        ensure_output(&output, overwrite)?;
    }
    let count = files.len();
    let mut total = 0;
    for (source, rel) in files {
        let destination = if input.is_file() {
            output.clone()
        } else {
            output.join(&rel).with_extension("json")
        };
        total += oretubar_tob_sob_tool::sob::extract_file(
            &source,
            &destination,
            rel.to_string_lossy().as_ref(),
            encoding,
            overwrite,
        )?;
    }
    println!("Extracted {total} entries from {count} SOB files.");
    Ok(())
}

fn interactive_inject(prefill: Option<PathBuf>) -> Result<()> {
    let default_input = prefill.as_deref().and_then(Path::to_str);
    let input = PathBuf::from(prompt("Original SOB file/directory", default_input)?);
    let translation = PathBuf::from(prompt(
        "Translation JSON file/directory",
        Some("translation_json"),
    )?);
    let default_output = if input.is_file() {
        "rebuilt.sob"
    } else {
        "rebuilt"
    };
    let output = PathBuf::from(prompt(
        "Rebuilt output file/directory",
        Some(default_output),
    )?);
    let encoding = EncodingChoice::parse(&prompt("Source encoding (sjis/gbk)", None)?)?;
    let overwrite = if output.exists() {
        confirm("Output exists; overwrite it")?
    } else {
        false
    };
    if output.exists() && !overwrite {
        println!("Cancelled.");
        return Ok(());
    }
    println!(
        "Inject {} + {} -> {} ({})",
        input.display(),
        translation.display(),
        output.display(),
        encoding.label()
    );
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    let files = collect_files(&input, "sob")?;
    if input.is_dir() {
        ensure_output(&output, overwrite)?;
    }
    let mut changed = 0;
    for (source, rel) in files {
        let trans = if translation.is_file() {
            translation.clone()
        } else {
            json_path(&translation, &rel)
        };
        if !trans.exists() {
            return Err(format!("missing translation {}", trans.display()));
        }
        let destination = if input.is_file() {
            output.clone()
        } else {
            output.join(&rel)
        };
        changed += oretubar_tob_sob_tool::sob::inject_file(
            &source,
            &trans,
            &destination,
            encoding,
            overwrite,
        )?;
    }
    println!("Updated {changed} SOB records.");
    Ok(())
}

fn interactive_repair(prefill: Option<PathBuf>) -> Result<()> {
    let default_input = prefill.as_deref().and_then(Path::to_str);
    let input = PathBuf::from(prompt("CHS SOB file/directory", default_input)?);
    let baseline = PathBuf::from(prompt("ROW SOB file/directory", None)?);
    let output = PathBuf::from(prompt("Repaired output file/directory", Some("repaired"))?);
    let report = PathBuf::from(prompt(
        "Unresolved report JSON",
        Some("offset-repair-report.json"),
    )?);
    let needs_overwrite = output.exists() || report.exists();
    let overwrite = if needs_overwrite {
        confirm("Output or report exists; overwrite")?
    } else {
        false
    };
    if needs_overwrite && !overwrite {
        println!("Cancelled.");
        return Ok(());
    }
    println!(
        "Repair {} using {} -> {} (report {})",
        input.display(),
        baseline.display(),
        output.display(),
        report.display()
    );
    if !confirm("Proceed")? {
        println!("Cancelled.");
        return Ok(());
    }
    run_repair(RepairArgs {
        input,
        baseline,
        output,
        report,
        overwrite,
    })
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
