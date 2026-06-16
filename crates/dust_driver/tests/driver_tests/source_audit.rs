use std::{fs, path::Path};

#[test]
fn production_code_uses_shared_annotation_accessors() {
    let root = workspace_root();
    let mut violations = Vec::new();

    for dir in [
        "crates/dust_db_plugin/src",
        "crates/dust_plugin_derive/src",
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

#[test]
fn tree_sitter_nodes_stay_backend_private() {
    let root = workspace_root();
    let mut violations = Vec::new();

    for entry in fs::read_dir(root.join("crates")).expect("crates directory exists") {
        let path = entry.expect("crate entry is readable").path();
        if path.file_name().and_then(|name| name.to_str()) == Some("dust_parser_dart_ts") {
            continue;
        }
        scan_dir(
            &path.join("src"),
            &tree_sitter_public_boundary_patterns(),
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "tree-sitter nodes must stay private to dust_parser_dart_ts:\n{}",
        violations.join("\n")
    );
}

#[test]
fn compatibility_aliases_stay_in_canonical_modules() {
    let root = workspace_root();
    let mut violations = Vec::new();

    assert_alias_defined_once(
        &root.join("crates/dust_ir/src/library.rs"),
        "pub type LibraryIr = DartFileIr;",
    );
    assert_alias_defined_once(
        &root.join("crates/dust_parser_dart/src/surface.rs"),
        "pub type ParsedLibrarySurface = ParsedDartFileSurface;",
    );

    for entry in fs::read_dir(root.join("crates")).expect("crates directory exists") {
        let path = entry.expect("crate entry is readable").path();
        scan_dir_with_exclusions(
            &path.join("src"),
            &compatibility_alias_patterns(),
            &[
                root.join("crates/dust_ir/src/library.rs"),
                root.join("crates/dust_parser_dart/src/surface.rs"),
            ],
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "compatibility aliases must only be defined at their canonical shims:\n{}",
        violations.join("\n")
    );
}

#[test]
fn production_code_uses_canonical_file_model_names() {
    let root = workspace_root();
    let mut violations = Vec::new();

    for entry in fs::read_dir(root.join("crates")).expect("crates directory exists") {
        let path = entry.expect("crate entry is readable").path();
        scan_dir_with_exclusions(
            &path.join("src"),
            &["LibraryIr", "ParsedLibrarySurface"],
            &[
                root.join("crates/dust_ir/src/library.rs"),
                root.join("crates/dust_ir/src/lib.rs"),
                root.join("crates/dust_parser_dart/src/surface.rs"),
                root.join("crates/dust_parser_dart/src/lib.rs"),
            ],
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "production code must use DartFileIr and ParsedDartFileSurface outside compatibility shims:\n{}",
        violations.join("\n")
    );
}

#[test]
fn db_query_discovery_uses_tree_sitter_nodes() {
    let root = workspace_root();
    let mut violations = Vec::new();

    scan_file(
        &root.join("crates/dust_parser_dart_ts/src/queries.rs"),
        &[
            "source.as_str().find",
            "collect_calls(source.as_str",
            "is_code_position",
            "is_identifier_boundary",
        ],
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "DB query discovery must use tree-sitter nodes instead of source scanning:\n{}",
        violations.join("\n")
    );
}

#[test]
fn parser_type_extraction_uses_tree_sitter_nodes() {
    let root = workspace_root();
    let mut violations = Vec::new();

    scan_dir(
        &root.join("crates/dust_parser_dart_ts/src/classes"),
        &[
            "extract_type_prefix",
            "extract_parameter_type",
            "strip_prefix_modifiers",
            "strip_leading_annotations",
            "rfind(name)",
        ],
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "Dart type extraction must use tree-sitter type nodes instead of declaration-prefix parsing:\n{}",
        violations.join("\n")
    );
}

#[test]
fn parser_declaration_names_use_grammar_fields() {
    let root = workspace_root();
    let mut violations = Vec::new();

    scan_dir(
        &root.join("crates/dust_parser_dart_ts/src/classes"),
        &[
            "find_last_descendant",
            "find_last_descendant_text",
            "collect_descendants",
        ],
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "Dart declaration names must use grammar fields or direct children instead of last-descendant identifier guessing:\n{}",
        violations.join("\n")
    );
}

#[test]
fn parser_class_modifiers_use_grammar_tokens() {
    let root = workspace_root();
    let mut violations = Vec::new();

    scan_dir(
        &root.join("crates/dust_parser_dart_ts/src"),
        &["class_header_text"],
        &mut violations,
    );
    scan_file(
        &root.join("crates/dust_parser_dart_ts/src/classes/class_decl.rs"),
        &["header.contains", "split_whitespace"],
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "Class modifiers must use tree-sitter modifier tokens instead of header source parsing:\n{}",
        violations.join("\n")
    );
}

#[test]
fn parser_method_modifiers_and_bodies_use_tree_sitter_nodes() {
    let root = workspace_root();
    let mut violations = Vec::new();

    scan_file(
        &root.join("crates/dust_parser_dart_ts/src/classes/methods.rs"),
        &[
            "header_text",
            "declaration_text",
            "after_params",
            "contains(\"static\")",
            "contains(\"external\")",
            "contains('{')",
            "contains(\"=>\")",
        ],
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "Method modifiers and bodies must use tree-sitter tokens/body nodes instead of source parsing:\n{}",
        violations.join("\n")
    );
}

#[test]
fn parser_constructor_redirection_uses_tree_sitter_fields() {
    let root = workspace_root();
    let mut violations = Vec::new();

    scan_dir(
        &root.join("crates/dust_parser_dart_ts/src/classes"),
        &[
            "extract_redirect_target",
            "extract_redirect_target_name",
            "split_once('=')",
            "chars().peekable()",
        ],
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "Constructor redirection targets must use tree-sitter target fields instead of source scanning:\n{}",
        violations.join("\n")
    );
}

#[test]
fn parser_defaults_use_tree_sitter_expression_nodes() {
    let root = workspace_root();
    let mut violations = Vec::new();

    scan_dir(
        &root.join("crates/dust_parser_dart_ts/src/classes"),
        &[
            "extract_default_value_source",
            "trailing_default_value_source",
            "top_level_default_end",
            "top_level_equals_index",
            "contains('=')",
        ],
        &mut violations,
    );

    assert!(
        violations.is_empty(),
        "Default values must use tree-sitter value/expression nodes instead of source scanning:\n{}",
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
        "derive_member_names",
        "derive_arguments_has_copy_with",
        "source.find(\"@",
        "parse_serde_arguments",
    ]
}

fn dart_source_parser_patterns() -> Vec<&'static str> {
    vec![
        "struct DelimiterState",
        "fn balanced_parenthesized",
        "fn find_top_level_char",
        "fn has_top_level_char",
        "fn split_top_level_items",
        "fn split_top_level_once",
        "fn normalized_args",
        "fn parse_named_arguments",
        "fn parse_string_literal",
        "fn parse_bool_literal",
        "fn parse_static_dart_string_literal",
        "fn parse_member_ref",
        "fn parse_type_name",
        "fn parse_string_list",
        "fn parse_type_list",
        "fn parse_constructor_name",
        "fn parse_constructor_list",
        "fn parse_string_map",
    ]
}

fn tree_sitter_public_boundary_patterns() -> Vec<&'static str> {
    vec![
        concat!("tree_sitter", "::"),
        concat!("use tree_sitter", "::"),
        concat!("Node", "<'"),
    ]
}

fn compatibility_alias_patterns() -> Vec<&'static str> {
    vec!["pub type LibraryIr", "pub type ParsedLibrarySurface"]
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

fn scan_dir_with_exclusions(
    dir: &Path,
    patterns: &[&str],
    excluded_files: &[std::path::PathBuf],
    violations: &mut Vec<String>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if excluded_files.iter().any(|excluded| excluded == &path) {
            continue;
        }
        if path.is_dir() {
            scan_dir_with_exclusions(&path, patterns, excluded_files, violations);
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

fn assert_alias_defined_once(path: &Path, alias: &str) {
    let source = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("{} should be readable: {error}", path.display());
    });
    let count = source.match_indices(alias).count();
    assert_eq!(
        count,
        1,
        "{} should define `{alias}` exactly once",
        path.display()
    );
}
