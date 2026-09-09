//! Plugin boundaries: shared analysis, the generate API and the tool fingerprint.

use super::*;

#[test]
fn production_plugins_use_canonical_ir_workspace_analysis() {
    let root = workspace_root();
    let mut violations = Vec::new();

    for dir in [
        "crates/dust_plugin_api/src",
        "crates/dust_plugin_derive/src",
        "crates/dust_plugin_serde/src",
        "crates/dust_http_client_plugin/src",
        "crates/dust_route_plugin/src",
        "crates/dust_state_plugin/src",
        "crates/dust_db_plugin/src",
    ] {
        scan_dir(
            &root.join(dir),
            &[
                "ParsedDartFileSurface",
                "WorkspaceAnalysisContext",
                "collect_workspace_analysis(",
            ],
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "production plugins must collect workspace facts from canonical DartFileIr:\n{}",
        violations.join("\n")
    );
}

#[test]
fn production_plugins_use_generate_api() {
    let root = workspace_root();
    let mut violations = Vec::new();

    for dir in [
        "crates/dust_plugin_api/src",
        "crates/dust_plugin_derive/src",
        "crates/dust_plugin_serde/src",
        "crates/dust_http_client_plugin/src",
        "crates/dust_route_plugin/src",
        "crates/dust_state_plugin/src",
        "crates/dust_db_plugin/src",
        "crates/dust_driver/src",
    ] {
        scan_dir(
            &root.join(dir),
            &["fn emit(", "GeneratedUnit", "emit_contributions"],
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "production plugins must use the generate API without legacy emit adapters:\n{}",
        violations.join("\n")
    );
}

#[test]
fn workspace_analysis_uses_canonical_ir_before_emission() {
    let root = workspace_root();
    let scan = fs::read_to_string(root.join("crates/dust_driver/src/build/process/scan.rs"))
        .expect("workspace scan source should be readable");
    let lower = fs::read_to_string(root.join("crates/dust_driver/src/lower.rs"))
        .expect("lowering source should be readable");
    let execute = fs::read_to_string(root.join("crates/dust_driver/src/build/process/execute.rs"))
        .expect("library execution source should be readable");

    assert!(
        scan.contains("collect_workspace_analysis_ir"),
        "workspace scan must dispatch plugin analysis from canonical IR"
    );
    assert!(
        !scan.contains("registry.collect_workspace_analysis("),
        "workspace scan must not dispatch parser-only workspace analysis"
    );
    assert!(
        !lower.contains(".filter(|class| required_classes.contains"),
        "canonical IR workspace analysis must retain unannotated declarations"
    );
    assert!(
        scan.contains("lower_for_workspace_analysis"),
        "workspace scan must lower libraries before collecting IR analysis"
    );
    assert!(
        execute.contains("pre_lowered"),
        "library execution must reuse the IR lowered during workspace analysis"
    );
}

#[test]
fn driver_lowering_uses_resolver_diagnostic_scope() {
    let root = workspace_root();
    let lower = fs::read_to_string(root.join("crates/dust_driver/src/lower.rs"))
        .expect("lowering source should be readable");

    assert!(
        lower.contains("requires_lowering_diagnostics"),
        "driver lowering must consume resolver-owned diagnostic scope"
    );
    assert!(
        !lower.contains("dust_dart::SerDe"),
        "driver lowering must not branch on SerDe symbols"
    );
    assert!(
        !lower.contains("tryFrom"),
        "driver lowering must not parse DB tryFrom configuration"
    );
}

#[test]
fn feature_plugins_do_not_parse_raw_dart_sources() {
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
            &[
                "source.as_str().find",
                "source.find(\"@",
                "parse_named_arguments",
                "parse_type_name",
                "split_top_level_items",
                "split_top_level_once",
            ],
            &mut violations,
        );
    }

    assert!(
        violations.is_empty(),
        "feature plugins must consume parser/IR facts instead of raw Dart parsing:\n{}",
        violations.join("\n")
    );
}

#[test]
fn codegen_tool_fingerprint_is_generated_from_source_roots() {
    let root = workspace_root();
    let build_script = fs::read_to_string(root.join("crates/dust_driver/build.rs"))
        .expect("dust_driver build script should be readable");
    let tool_hash_source =
        fs::read_to_string(root.join("crates/dust_driver/src/build/support/tool_hash.rs"))
            .expect("tool hash source should be readable");

    assert!(
        tool_hash_source.contains("env!(\"OUT_DIR\")")
            && tool_hash_source.contains("codegen_tool_fingerprint.txt"),
        "tool hash must include the generated fingerprint manifest"
    );
    assert!(
        !tool_hash_source.contains("dust_plugin_derive/src/features/debug.rs"),
        "tool hash must not return to manual per-file fingerprint registration"
    );

    for root in [
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
    ] {
        assert!(
            build_script.contains(root),
            "build script must fingerprint {root}"
        );
    }

    for behavior in [
        "collect_fingerprint_files",
        "cargo:rerun-if-changed",
        "is_fingerprint_file",
        "tests.rs",
    ] {
        assert!(
            build_script.contains(behavior),
            "build script must preserve fingerprint behavior `{behavior}`"
        );
    }
}
