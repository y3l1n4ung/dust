use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve repo root")
}

fn benchmark_project_root() -> PathBuf {
    repo_root().join("examples/benchmark_project")
}

fn release_dust_binary(repo: &Path) -> PathBuf {
    if let Some(configured) = env::var_os("DUST_PERF_CLI_BIN") {
        let path = PathBuf::from(configured);
        return if path.is_absolute() {
            path
        } else {
            repo.join(path)
        };
    }

    run(Command::new("cargo")
        .args(["build", "-q", "-p", "dust_cli", "--release"])
        .current_dir(repo));

    repo.join("target")
        .join("release")
        .join(format!("dust{}", env::consts::EXE_SUFFIX))
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
    parse_build_metric(stdout, "time")
}

fn parse_build_metric(stdout: &str, key: &str) -> u64 {
    let line = stdout
        .lines()
        .find(|line| line.starts_with("build  "))
        .expect("build summary line");
    line.split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|window| {
            let token = window[0].strip_suffix(':')?;
            if token != key {
                return None;
            }
            let value = window[1].strip_suffix("ms").unwrap_or(window[1]);
            value.parse::<u64>().ok()
        })
        .unwrap_or_else(|| panic!("missing `{key}` metric in build summary: {line}"))
}

fn max_ms(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn invalidate_tool_hash(cache_path: &Path) {
    let contents = fs::read_to_string(cache_path).expect("read build cache");
    let mut rewritten = String::with_capacity(contents.len());
    let mut rest = contents.as_str();
    const KEY: &str = "\"tool_hash\":";

    while let Some(index) = rest.find(KEY) {
        let (before, after_key_start) = rest.split_at(index + KEY.len());
        rewritten.push_str(before);

        let mut after_key = after_key_start;
        let whitespace_len = after_key
            .bytes()
            .take_while(u8::is_ascii_whitespace)
            .count();
        rewritten.push_str(&after_key[..whitespace_len]);
        after_key = &after_key[whitespace_len..];

        let number_len = after_key.bytes().take_while(u8::is_ascii_digit).count();
        rewritten.push('0');
        rest = &after_key[number_len..];
    }

    rewritten.push_str(rest);
    fs::write(cache_path, rewritten).expect("write invalidated build cache");
}

#[test]
#[ignore = "expensive release perf regression test"]
fn benchmark_project_release_build_benchmark() {
    let repo = repo_root();
    let benchmark = benchmark_project_root();
    let benchmark_str = benchmark.to_string_lossy().into_owned();
    let dust = release_dust_binary(&repo);

    run(Command::new("flutter")
        .arg("pub")
        .arg("get")
        .current_dir(&repo));
    run(Command::new("flutter")
        .arg("pub")
        .arg("get")
        .current_dir(&benchmark));
    run(Command::new("bash")
        .arg("./generate.sh")
        .arg("--count")
        .arg("5000")
        .current_dir(&benchmark));
    run(Command::new(&dust)
        .args(["clean", "--root"])
        .arg(&benchmark_str)
        .current_dir(&repo));

    let cold = run(Command::new(&dust)
        .args(["build", "--root"])
        .arg(&benchmark_str)
        .current_dir(&repo));
    let warm = run(Command::new(&dust)
        .args(["build", "--root"])
        .arg(&benchmark_str)
        .current_dir(&repo));
    invalidate_tool_hash(&benchmark.join(".dart_tool/dust/build_cache_v1.json"));
    let invalidated = run(Command::new(&dust)
        .args(["build", "--root"])
        .arg(&benchmark_str)
        .arg("--fail-fast")
        .current_dir(&repo));

    let cold_ms = parse_build_time_ms(&cold);
    let warm_ms = parse_build_time_ms(&warm);
    let invalidated_ms = parse_build_time_ms(&invalidated);
    let cold_scanned = parse_build_metric(&cold, "scanned");
    let warm_scanned = parse_build_metric(&warm, "scanned");
    let cold_generated = parse_build_metric(&cold, "generated");
    let invalidated_scanned = parse_build_metric(&invalidated, "scanned");
    let cold_max = max_ms("DUST_PERF_COLD_MAX_MS", 2_000);
    let warm_max = max_ms("DUST_PERF_WARM_MAX_MS", 800);
    let invalidated_max = max_ms("DUST_PERF_INVALIDATED_MAX_MS", 1_400);

    assert!(
        cold_scanned >= 5_000,
        "cold build scanned too few files: {cold_scanned}\n{cold}"
    );
    assert!(
        warm_scanned >= 5_000,
        "warm build scanned too few files: {warm_scanned}\n{warm}"
    );
    assert!(
        cold_generated >= 5_000,
        "cold build generated too few files: {cold_generated}\n{cold}"
    );
    assert!(
        invalidated_scanned >= 5_000,
        "invalidated build scanned too few files: {invalidated_scanned}\n{invalidated}"
    );
    assert!(
        cold_ms <= cold_max,
        "cold build too slow: {cold_ms}ms > {cold_max}ms\n{cold}"
    );
    assert!(
        warm_ms <= warm_max,
        "warm build too slow: {warm_ms}ms > {warm_max}ms\n{warm}"
    );
    assert!(
        invalidated_ms <= invalidated_max,
        "invalidated build too slow: {invalidated_ms}ms > {invalidated_max}ms\n{invalidated}"
    );
}
