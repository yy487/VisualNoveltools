use std::env;
use std::path::PathBuf;

use qc_keisei_tools::cli;

const HELP: &str = "Adviz for Windows95 (WADVIZ) ADV text injector\n\nUsage:\n  qc_inject.exe [--output PATH] SOURCE.ADV SOURCE.ADV.json\n  qc_inject.exe [--output DIRECTORY] SOURCE_DIRECTORY JSON_DIRECTORY\n  qc_inject.exe [--output PATH] SOURCE_OR_JSON\n\nDrag and drop:\n  Drop TEXT_json alone to find sibling TEXT automatically.\n  Drop TEXT alone to find sibling TEXT_json automatically.\n  Single-file A01.ADV and A01.ADV.json paths are inferred the same way.\n\nDefaults:\n  A01.ADV + A01.ADV.json -> A01_injected.ADV\n  TEXT/ + TEXT_json/      -> TEXT_injected/\n\nThe output must not already exist. Directory mode copies the full source tree and patches matching JSON files.\n";

fn strip_ascii_suffix(path: &std::path::Path, suffix: &str) -> Result<Option<PathBuf>, String> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("path has no valid Unicode file name: {}", path.display()))?;
    if !name.to_ascii_lowercase().ends_with(suffix) {
        return Ok(None);
    }
    Ok(Some(
        path.with_file_name(&name[..name.len() - suffix.len()]),
    ))
}

fn append_to_name(path: &std::path::Path, suffix: &str) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .ok_or_else(|| format!("path has no file name: {}", path.display()))?;
    let mut output_name = name.to_os_string();
    output_name.push(suffix);
    Ok(path.with_file_name(output_name))
}

fn looks_like_translation(path: &std::path::Path) -> Result<bool, String> {
    Ok(strip_ascii_suffix(path, ".adv.json")?.is_some()
        || strip_ascii_suffix(path, "_json")?.is_some())
}

fn infer_single(path: PathBuf) -> Result<(PathBuf, PathBuf), String> {
    if let Some(source) = strip_ascii_suffix(&path, ".json")? {
        if source
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("adv"))
        {
            return Ok((source, path));
        }
    }
    if let Some(source) = strip_ascii_suffix(&path, "_json")? {
        return Ok((source, path));
    }
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("adv"))
    {
        let translations = append_to_name(&path, ".json")?;
        return Ok((path, translations));
    }
    let translations = append_to_name(&path, "_json")?;
    Ok((path, translations))
}

fn resolve_inputs(mut positional: Vec<PathBuf>) -> Result<(PathBuf, PathBuf), String> {
    match positional.len() {
        1 => infer_single(positional.remove(0)),
        2 => {
            let first = positional.remove(0);
            let second = positional.remove(0);
            if looks_like_translation(&first)? && !looks_like_translation(&second)? {
                Ok((second, first))
            } else {
                Ok((first, second))
            }
        }
        _ => Err("expected one inferred path or explicit SOURCE and JSON paths".to_owned()),
    }
}

fn parse_args() -> Result<(PathBuf, PathBuf, Option<PathBuf>), String> {
    let mut positional = Vec::new();
    let mut output = None;
    let mut args = env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "-h" || argument == "--help" {
            print!("{HELP}");
            std::process::exit(0);
        }
        if argument == "--output" || argument == "-o" {
            let value = args
                .next()
                .ok_or_else(|| "--output requires a path".to_owned())?;
            if output.replace(PathBuf::from(value)).is_some() {
                return Err("--output may only be specified once".to_owned());
            }
            continue;
        }
        positional.push(PathBuf::from(argument));
    }
    let (source, translations) = resolve_inputs(positional)?;
    Ok((source, translations, output))
}

fn main() {
    let result = parse_args().and_then(|(source, translations, output)| {
        cli::inject(&source, &translations, output.as_deref())
            .map(|report| (source, translations, report))
    });
    match result {
        Ok((source, translations, report)) => {
            println!("[inject] source={}", source.display());
            println!("[inject] translations={}", translations.display());
            println!("[inject] json_entries={}", report.json_entries);
            println!("[inject] patched={}", report.patched);
            println!("[inject] unchanged={}", report.unchanged);
            println!("[inject] failed={}", report.failed);
            println!("[inject] warnings={}", report.warnings);
            println!("[inject] copied_files={}", report.copied_files);
            println!("[inject] output={}", report.output.display());
        }
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("\n{HELP}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_directory_drag_drop_in_both_directions() {
        assert_eq!(
            resolve_inputs(vec![PathBuf::from(r"E:\game\TEXT_JSON")]).unwrap(),
            (
                PathBuf::from(r"E:\game\TEXT"),
                PathBuf::from(r"E:\game\TEXT_JSON")
            )
        );
        assert_eq!(
            resolve_inputs(vec![PathBuf::from(r"E:\game\TEXT")]).unwrap(),
            (
                PathBuf::from(r"E:\game\TEXT"),
                PathBuf::from(r"E:\game\TEXT_json")
            )
        );
    }

    #[test]
    fn infers_single_file_drag_drop_in_both_directions() {
        assert_eq!(
            resolve_inputs(vec![PathBuf::from(r"E:\game\A01.ADV.json")]).unwrap(),
            (
                PathBuf::from(r"E:\game\A01.ADV"),
                PathBuf::from(r"E:\game\A01.ADV.json")
            )
        );
        assert_eq!(
            resolve_inputs(vec![PathBuf::from(r"E:\game\A01.ADV")]).unwrap(),
            (
                PathBuf::from(r"E:\game\A01.ADV"),
                PathBuf::from(r"E:\game\A01.ADV.json")
            )
        );
    }

    #[test]
    fn accepts_reverse_order_when_two_paths_are_dropped() {
        assert_eq!(
            resolve_inputs(vec![
                PathBuf::from(r"E:\game\TEXT_json"),
                PathBuf::from(r"E:\game\TEXT"),
            ])
            .unwrap(),
            (
                PathBuf::from(r"E:\game\TEXT"),
                PathBuf::from(r"E:\game\TEXT_json")
            )
        );
    }
}
