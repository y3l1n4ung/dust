use std::{fs, path::Path};

#[test]
fn production_code_uses_shared_annotation_accessors() {
    let root = workspace_root();
    let mut violations = Vec::new();

    for dir in [
        "crates/dust_db_plugin/src",
        "crates/dust_http_client_plugin/src",
        "crates/dust_route_plugin/src",
        "crates/dust_state_plugin/src",
    ] {
        scan_dir(
            &root.join(dir),
            &plugin_forbidden_patterns(),
            &mut violations,
        );
    }

    scan_dir(
        &root.join("crates/dust_driver/src"),
        &driver_forbidden_patterns(),
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "production code must use parser/IR annotation accessors:\n{}",
        violations.join("\n")
    );
}

#[test]
fn dart_source_parsing_helpers_are_centralized() {
    let root = workspace_root();
    let mut violations = Vec::new();

    for entry in fs::read_dir(root.join("crates")).expect("crates directory exists") {
        let path = entry.expect("crate entry is readable").path();
        if path.file_name().and_then(|name| name.to_str()) == Some("dust_dart_syntax") {
            continue;
        }
        scan_dir(
            &path.join("src"),
            &dart_source_parser_patterns(),
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "Dart source parsing helpers must live in dust_dart_syntax:\n{}",
        violations.join("\n")
    );
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("dust_driver lives under crates/dust_driver")
}

fn plugin_forbidden_patterns() -> Vec<&'static str> {
    let mut patterns = driver_forbidden_patterns();
    patterns.extend([
        "normalized_args",
        "parse_named_arguments",
        "split_top_level_items",
        "split_top_level_once",
    ]);
    patterns
}

fn driver_forbidden_patterns() -> Vec<&'static str> {
    vec![
        "arguments_source.as_deref",
        "source.find(\"@",
        "parse_serde_arguments",
    ]
}

fn dart_source_parser_patterns() -> Vec<&'static str> {
    vec![
        "struct DelimiterState",
        "fn split_top_level_items",
        "fn split_top_level_once",
        "fn parse_string_literal",
        "fn parse_bool_literal",
    ]
}

fn scan_dir(dir: &Path, patterns: &[&str], violations: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, patterns, violations);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        scan_file(&path, patterns, violations);
    }
}

fn scan_file(path: &Path, patterns: &[&str], violations: &mut Vec<String>) {
    let Ok(source) = fs::read_to_string(path) else {
        return;
    };
    for (line_index, line) in source.lines().enumerate() {
        for pattern in patterns {
            if line.contains(pattern) {
                violations.push(format!(
                    "{}:{} contains `{}`",
                    path.display(),
                    line_index + 1,
                    pattern
                ));
            }
        }
    }
}
