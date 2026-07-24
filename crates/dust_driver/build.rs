//! Builds a deterministic manifest of Dust codegen source inputs.

use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

/// Generated manifest filename included by `tool_hash.rs`.
const FINGERPRINT_MANIFEST: &str = "codegen_tool_fingerprint.txt";

/// Source roots that can affect generated Dart output.
const CODEGEN_FINGERPRINT_ROOTS: &[&str] = &[
    "crates/dust_dart_emit/src",
    "crates/dust_dart_syntax/src",
    "crates/dust_db_plugin/src",
    "crates/dust_driver/src",
    "crates/dust_emitter/src",
    "crates/dust_http_client_plugin/src",
    "crates/dust_ir/src",
    "crates/dust_parser_dart/src",
    "crates/dust_parser_dart_ts/src",
    "crates/dust_plugin_api/src",
    "crates/dust_plugin_derive/src",
    "crates/dust_plugin_serde/src",
    "crates/dust_resolver/src",
    "crates/dust_route_plugin/src",
    "crates/dust_state_plugin/src",
    "crates/dust_workspace/src",
];

/// Generates the codegen fingerprint manifest consumed by the build cache.
fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let repo_root = repo_root();
    let mut files = Vec::new();
    for relative_root in CODEGEN_FINGERPRINT_ROOTS {
        let root = repo_root.join(relative_root);
        collect_fingerprint_files(&root, &mut files)?;
    }
    files.sort();

    let manifest = build_fingerprint_manifest(&repo_root, &files)?;
    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .expect("Cargo must set OUT_DIR for build scripts");
    fs::write(out_dir.join(FINGERPRINT_MANIFEST), manifest)
}

/// Returns the repository root from the `dust_driver` crate root.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("dust_driver must be inside the crates directory")
        .to_path_buf()
}

/// Recursively collects source and template files for fingerprinting.
fn collect_fingerprint_files(root: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", root.display());
    let entries = fs::read_dir(root)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if !is_excluded_dir(&path) {
                collect_fingerprint_files(&path, files)?;
            }
        } else if file_type.is_file() && is_fingerprint_file(&path) {
            println!("cargo:rerun-if-changed={}", path.display());
            files.push(path);
        }
    }

    Ok(())
}

/// Returns whether a directory should be skipped.
fn is_excluded_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.') || matches!(name, "benches" | "tests"))
}

/// Returns whether a file should be fingerprinted.
fn is_fingerprint_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    !file_name.starts_with('.') && file_name != "tests.rs" && !file_name.starts_with("tests_")
}

/// Builds the deterministic manifest from sorted absolute paths.
fn build_fingerprint_manifest(repo_root: &Path, files: &[PathBuf]) -> io::Result<String> {
    let mut manifest = String::from("dust-codegen-fingerprint-v1\n");
    for path in files {
        let relative = path
            .strip_prefix(repo_root)
            .expect("fingerprint file must be under repo root");
        let bytes = fs::read(path)?;
        manifest.push_str(&manifest_path(relative));
        manifest.push('\0');
        manifest.push_str(&bytes.len().to_string());
        manifest.push('\0');
        manifest.push_str(&fnv1a_hex(&bytes));
        manifest.push('\n');
    }
    Ok(manifest)
}

/// Converts a path to a stable manifest path.
fn manifest_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Hashes file bytes with the same stable FNV-1a family used by Dust caches.
fn fnv1a_hex(bytes: &[u8]) -> String {
    let mut hash = 1469598103934665603_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}
