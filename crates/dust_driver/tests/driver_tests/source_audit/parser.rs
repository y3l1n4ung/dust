//! Parser boundaries: every backend detail stays behind tree-sitter.

use super::*;

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
