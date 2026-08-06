use kokorov_fl2::ab::{parse_script, AbStringKind};
use kokorov_fl2::{ToolError, ToolResult};
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

fn usage(program: &str) {
    println!(
        "KOKOROV AB bytecode inspector\n\nUsage:\n  {program} INPUT\n\nINPUT may be one .AB file or a directory containing .AB files. The inspector\nvalidates instruction boundaries, CP932 strings, and absolute branch targets.\nIt never writes files."
    );
}

fn collect_inputs(input: &Path) -> ToolResult<Vec<PathBuf>> {
    if input.is_file() {
        return Ok(vec![input.to_path_buf()]);
    }
    if !input.is_dir() {
        return Err(ToolError(format!(
            "input is not a file or directory: '{}'",
            input.display()
        )));
    }
    let mut paths = std::fs::read_dir(input)
        .map_err(|error| ToolError(format!("cannot read '{}': {error}", input.display())))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("ab"))
        })
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_ascii_uppercase())
            .unwrap_or_default()
    });
    if paths.is_empty() {
        return Err(ToolError(format!(
            "directory contains no .AB files: '{}'",
            input.display()
        )));
    }
    Ok(paths)
}

fn run() -> ToolResult<()> {
    let mut args = env::args_os();
    let program = args
        .next()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "ab_inspect".to_string());
    let input = match args.next() {
        Some(value) if value == "-h" || value == "--help" => {
            usage(&program);
            return Ok(());
        }
        Some(value) => PathBuf::from(value),
        None => {
            usage(&program);
            return Err(ToolError("missing INPUT".to_string()));
        }
    };
    if args.next().is_some() {
        return Err(ToolError("only one INPUT is supported".to_string()));
    }

    let paths = collect_inputs(&input)?;
    let mut instructions = 0usize;
    let mut messages = 0usize;
    let mut choices = 0usize;
    let mut buffers = 0usize;
    let mut targets = 0usize;
    let mut opcodes = BTreeMap::<u16, usize>::new();
    for path in &paths {
        let bytes = std::fs::read(path)
            .map_err(|error| ToolError(format!("cannot read '{}': {error}", path.display())))?;
        let script = parse_script(&bytes)
            .map_err(|error| ToolError(format!("{}: {error}", path.display())))?;
        instructions += script.instructions.len();
        for instruction in &script.instructions {
            *opcodes.entry(instruction.opcode).or_default() += 1;
            targets += instruction.targets.len();
            for string in &instruction.strings {
                match string.kind {
                    AbStringKind::Message => messages += 1,
                    AbStringKind::Choice => choices += 1,
                    AbStringKind::Buffer => buffers += 1,
                    AbStringKind::Resource => {}
                }
            }
        }
    }
    let opcode_report = opcodes
        .into_iter()
        .map(|(opcode, count)| format!("{opcode}:{count}"))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "[inspect] files={} instructions={} messages={} choices={} buffers={} targets={} opcodes={}",
        paths.len(), instructions, messages, choices, buffers, targets, opcode_report
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ab_inspect: error: {error}");
        std::process::exit(1);
    }
}
