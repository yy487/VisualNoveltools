use kokoro2_ab::ab::{parse_file, rebuild_script, AbFile, AbStringKind};
use kokoro2_ab::{ToolError, ToolResult};
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::path::{Path, PathBuf};

fn usage(program: &str) {
    println!(
        "Kokoro 2 AB bytecode inspector\n\nUsage:\n  {program} INPUT\n\nINPUT may be one .AB file or a directory containing .AB files. The inspector\nvalidates Kokoro 2 instruction boundaries, CP932 strings, absolute branch\ntargets, and a byte-exact no-change rebuild. It never writes files."
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

    fn collect(directory: &Path, paths: &mut Vec<PathBuf>) -> ToolResult<()> {
        let entries = std::fs::read_dir(directory).map_err(|error| {
            ToolError(format!("cannot read '{}': {error}", directory.display()))
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                ToolError(format!(
                    "cannot enumerate '{}': {error}",
                    directory.display()
                ))
            })?;
            let path = entry.path();
            if path.is_dir() {
                collect(&path, paths)?;
            } else if path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("ab"))
            {
                paths.push(path);
            }
        }
        Ok(())
    }

    let mut paths = Vec::new();
    collect(input, &mut paths)?;
    paths.sort_by_key(|path| {
        path.to_string_lossy()
            .replace('\\', "/")
            .to_ascii_uppercase()
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
    let mut bytecode_files = 0usize;
    let mut plain_cp932_files = 0usize;
    let mut plain_cp932_bytes = 0usize;
    let mut instructions = 0usize;
    let mut messages = 0usize;
    let mut choices = 0usize;
    let mut choice_prompts = 0usize;
    let mut buffers = 0usize;
    let mut resources = 0usize;
    let mut voices = 0usize;
    let mut targets = 0usize;
    let mut byte_exact_rebuilds = 0usize;
    let mut opcodes = BTreeMap::<u16, usize>::new();

    for path in &paths {
        let bytes = std::fs::read(path)
            .map_err(|error| ToolError(format!("cannot read '{}': {error}", path.display())))?;
        match parse_file(&bytes)
            .map_err(|error| ToolError(format!("{}: {error}", path.display())))?
        {
            AbFile::Bytecode(script) => {
                let rebuilt =
                    rebuild_script(&bytes, &script, &HashMap::new()).map_err(|error| {
                        ToolError(format!(
                            "{}: no-change rebuild failed: {error}",
                            path.display()
                        ))
                    })?;
                if rebuilt != bytes {
                    return Err(ToolError(format!(
                        "{}: no-change rebuild is not byte-exact",
                        path.display()
                    )));
                }
                byte_exact_rebuilds += 1;
                bytecode_files += 1;
                instructions += script.instructions.len();
                for instruction in &script.instructions {
                    *opcodes.entry(instruction.opcode).or_default() += 1;
                    targets += instruction.targets.len();
                    voices += usize::from(instruction.voice.is_some());
                    for string in &instruction.strings {
                        match string.kind {
                            AbStringKind::Message => messages += 1,
                            AbStringKind::Choice => choices += 1,
                            AbStringKind::ChoicePrompt => choice_prompts += 1,
                            AbStringKind::Buffer => buffers += 1,
                            AbStringKind::Resource => resources += 1,
                        }
                    }
                }
            }
            AbFile::PlainCp932(plain) => {
                plain_cp932_files += 1;
                plain_cp932_bytes += plain.raw.len();
            }
        }
    }

    let opcode_report = opcodes
        .into_iter()
        .map(|(opcode, count)| format!("{opcode}:{count}"))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "[inspect] files={} bytecode_files={} plain_cp932_files={} plain_cp932_bytes={}",
        paths.len(),
        bytecode_files,
        plain_cp932_files,
        plain_cp932_bytes
    );
    println!(
        "[inspect] instructions={} messages={} choices={} choice_prompts={} buffers={} resources={} voices={} targets={}",
        instructions, messages, choices, choice_prompts, buffers, resources, voices, targets
    );
    println!(
        "[inspect] byte_exact_rebuild_files={} opcodes={}",
        byte_exact_rebuilds, opcode_report
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ab_inspect: error: {error}");
        std::process::exit(1);
    }
}
