use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn real_shared_panel_and_help_are_reachable() {
    let exe = env!("CARGO_BIN_EXE_kohaku_pc98");
    let help = Command::new(exe).arg("--help").output().unwrap();
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("extract"));
    assert!(text.contains("unpack"));
    assert!(text.contains("export"));
    assert!(text.contains("import"));
    let mut panel = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    panel.stdin.take().unwrap().write_all(b"0\n").unwrap();
    let result = panel.wait_with_output().unwrap();
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("主菜单"));
    assert!(String::from_utf8_lossy(&result.stdout).contains("回注"));
    assert!(!Command::new(exe)
        .arg("extract")
        .output()
        .unwrap()
        .status
        .success());
}
