use std::process::Command;

#[test]
fn binary_help_smoke_test() {
    let binary = env!("CARGO_BIN_EXE_dust");
    let output = Command::new(binary).arg("--help").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Fast Dart code generation without build_runner"));
    assert!(stdout.contains("Usage: dust <COMMAND>"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("check"));
    assert!(stdout.contains("doctor"));
    assert!(stdout.contains("watch"));
    assert!(stdout.contains("--version"));
}

#[test]
fn binary_version_smoke_test() {
    let binary = env!("CARGO_BIN_EXE_dust");
    let output = Command::new(binary).arg("--version").output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, format!("dust {}\n", env!("CARGO_PKG_VERSION")));
}
