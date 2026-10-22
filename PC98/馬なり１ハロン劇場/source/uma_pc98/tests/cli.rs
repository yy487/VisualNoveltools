use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn command_and_interactive_entries_register_export_and_import() {
    let executable = env!("CARGO_BIN_EXE_uma_pc98");
    let help = Command::new(executable).arg("--help").output().unwrap();
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(
        text.contains("export")
            && text.contains("import")
            && text.contains("--workspace")
            && text.contains("--font")
            && text.contains("memory-fix")
            && text.contains("--hdi-base")
    );
    let mut panel = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    panel.stdin.take().unwrap().write_all(b"2\n0\n0\n").unwrap();
    let result = panel.wait_with_output().unwrap();
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("独立译文目录"));
}
