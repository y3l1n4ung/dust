use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repo root")
}

fn stress_project_root() -> PathBuf {
    repo_root().join("examples/stress_project")
}

fn run(command: &mut Command) -> String {
    let output = command.output().expect("run command");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    stdout
}

fn parse_build_time_ms(stdout: &str) -> u64 {
    let line = stdout
        .lines()
        .find(|line| line.starts_with("build  "))
        .expect("build summary line");
    let millis = line
        .split("time: ")
        .nth(1)
        .and_then(|tail| tail.strip_suffix("ms"))
        .expect("time suffix");
    millis.parse::<u64>().expect("parse build milliseconds")
}

fn max_ms(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

#[test]
#[ignore = "expensive release perf regression test"]
fn stress_project_release_build_benchmark() {
    let repo = repo_root();
    let stress = stress_project_root();
    let stress_str = stress.to_string_lossy().into_owned();

    run(Command::new("dart")
        .arg("pub")
        .arg("get")
        .current_dir(&stress));
    run(Command::new("bash")
        .arg("./generate.sh")
        .arg("--count")
        .arg("5000")
        .current_dir(&stress));
    run(Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "dust_cli",
            "--release",
            "--",
            "clean",
            "--root",
        ])
        .arg(&stress_str)
        .current_dir(&repo));

    let cold = run(Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "dust_cli",
            "--release",
            "--",
            "build",
            "--root",
        ])
        .arg(&stress_str)
        .current_dir(&repo));
    let warm = run(Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "dust_cli",
            "--release",
            "--",
            "build",
            "--root",
        ])
        .arg(&stress_str)
        .current_dir(&repo));

    let cold_ms = parse_build_time_ms(&cold);
    let warm_ms = parse_build_time_ms(&warm);
    let cold_max = max_ms("DUST_PERF_COLD_MAX_MS", 2_000);
    let warm_max = max_ms("DUST_PERF_WARM_MAX_MS", 800);

    assert!(cold.contains("scanned: 5000"));
    assert!(warm.contains("scanned: 5000"));
    assert!(
        cold_ms <= cold_max,
        "cold build too slow: {cold_ms}ms > {cold_max}ms\n{cold}"
    );
    assert!(
        warm_ms <= warm_max,
        "warm build too slow: {warm_ms}ms > {warm_max}ms\n{warm}"
    );
}
