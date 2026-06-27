use super::registry::RegistrySelection;

/// Stable hash of the active code generation logic and plugin set.
#[derive(Clone, Copy)]
pub(crate) struct CodegenToolHash {
    /// FNV-1a hash value used in cache entries.
    hash: u64,
}

impl CodegenToolHash {
    /// Returns the raw hash value persisted in cache metadata.
    pub(crate) fn value(self) -> u64 {
        self.hash
    }
}

/// Source inputs that affect core driver, resolver, workspace, and emitter behavior.
const CODEGEN_CORE_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../build.rs"),
    include_str!("../../check.rs"),
    include_str!("../../context.rs"),
    include_str!("../../watch.rs"),
    include_str!("../../lower.rs"),
    include_str!("../../lower/inheritance.rs"),
    include_str!("../../lower/parse_support.rs"),
    include_str!("../../lower/serde.rs"),
    include_str!("../../lower/serde_parse.rs"),
    include_str!("../../lower/type_parse.rs"),
    include_str!("../../../../dust_ir/src/annotation.rs"),
    include_str!("../../../../dust_ir/src/class.rs"),
    include_str!("../../../../dust_ir/src/constructor.rs"),
    include_str!("../../../../dust_ir/src/enum_type.rs"),
    include_str!("../../../../dust_ir/src/field.rs"),
    include_str!("../../../../dust_ir/src/lib.rs"),
    include_str!("../../../../dust_ir/src/library.rs"),
    include_str!("../../../../dust_ir/src/method.rs"),
    include_str!("../../../../dust_ir/src/serde.rs"),
    include_str!("../../../../dust_ir/src/traits.rs"),
    include_str!("../../../../dust_ir/src/types.rs"),
    include_str!("../../../../dust_resolver/src/annotations.rs"),
    include_str!("../../../../dust_resolver/src/catalog.rs"),
    include_str!("../../../../dust_resolver/src/lib.rs"),
    include_str!("../../../../dust_resolver/src/resolve.rs"),
    include_str!("../../../../dust_resolver/src/resolve_support.rs"),
    include_str!("../../../../dust_resolver/src/result.rs"),
    include_str!("../apply.rs"),
    include_str!("../batch.rs"),
    include_str!("../batch/load.rs"),
    include_str!("../batch/execute.rs"),
    include_str!("../process/mod.rs"),
    include_str!("../process/execute.rs"),
    include_str!("../process/output.rs"),
    include_str!("../process/scan.rs"),
    include_str!("../support.rs"),
    include_str!("cache_input.rs"),
    include_str!("registry.rs"),
    include_str!("tool_hash.rs"),
    include_str!("../work.rs"),
    include_str!("../../../../dust_plugin_api/src/analysis.rs"),
    include_str!("../../../../dust_plugin_api/src/contribution.rs"),
    include_str!("../../../../dust_plugin_api/src/plugin.rs"),
    include_str!("../../../../dust_plugin_api/src/registry.rs"),
    include_str!("../../../../dust_plugin_api/src/symbols.rs"),
    include_str!("../../../../dust_dart_emit/src/lib.rs"),
    include_str!("../../../../dust_dart_emit/src/rename.rs"),
    include_str!("../../../../dust_dart_emit/src/type_render.rs"),
    include_str!("../../../../dust_workspace/src/config.rs"),
    include_str!("../../../../dust_workspace/src/discover.rs"),
    include_str!("../../../../dust_workspace/src/output_policy.rs"),
    include_str!("../../../../dust_workspace/src/package_config.rs"),
    include_str!("../../../../dust_workspace/src/pubspec.rs"),
    include_str!("../../../../dust_workspace/src/root.rs"),
    include_str!("../../../../dust_workspace/src/workspace.rs"),
    include_str!("../../../../dust_emitter/src/emit.rs"),
    include_str!("../../../../dust_emitter/src/merge.rs"),
    include_str!("../../../../dust_emitter/src/write.rs"),
    include_str!("../../../../dust_emitter/src/writer.rs"),
);

/// Source inputs that affect generated derive behavior.
const DERIVE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../../dust_plugin_derive/src/analysis.rs"),
    include_str!("../../../../dust_plugin_derive/src/emit.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/mod.rs"),
    include_str!("../../../../dust_plugin_derive/src/plugin.rs"),
    include_str!("../../../../dust_plugin_derive/src/validate.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/debug.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/eq_hash.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/clone_copy_with.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/clone_copy_with/render.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/clone_copy_with/support.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/names.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/templates/debug_empty.jinja"),
    include_str!("../../../../dust_plugin_derive/src/features/templates/debug_fields.jinja"),
    include_str!("../../../../dust_plugin_derive/src/features/templates/eq_empty.jinja"),
    include_str!("../../../../dust_plugin_derive/src/features/templates/eq_fields.jinja"),
    include_str!("../../../../dust_plugin_derive/src/features/templates/hash_code.jinja"),
    include_str!("../../../../dust_plugin_derive/src/features/validate/mod.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/validate/emit.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/validate/model.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/validate/rule_snippets.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/validate/rules.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/validate/type_source.rs"),
    include_str!(
        "../../../../dust_plugin_derive/src/features/validate/templates/validate_mixin.jinja"
    ),
    include_str!(
        "../../../../dust_plugin_derive/src/features/validate/templates/validate_support.jinja"
    ),
    include_str!("../../../../dust_plugin_derive/src/features/writer.rs"),
    include_str!("../../../../dust_plugin_derive/src/lib.rs"),
);

/// Source inputs that affect generated SerDe behavior.
const SERDE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../../dust_plugin_serde/src/analysis.rs"),
    include_str!("../../../../dust_plugin_serde/src/lib.rs"),
    include_str!("../../../../dust_plugin_serde/src/plugin.rs"),
    include_str!("../../../../dust_plugin_serde/src/validate.rs"),
    include_str!("../../../../dust_plugin_serde/src/validate/json_capability.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit_class.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit_enum.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit_sealed.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit_support.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit_variant_class.rs"),
    include_str!("../../../../dust_plugin_serde/src/writer.rs"),
    include_str!("../../../../dust_plugin_serde/src/writer_expr.rs"),
    include_str!("../../../../dust_plugin_serde/src/writer_model.rs"),
    include_str!("../../../../dust_plugin_serde/src/writer_type.rs"),
    include_str!("../../../../dust_plugin_serde/src/templates/enum_from_json.jinja"),
    include_str!("../../../../dust_plugin_serde/src/templates/enum_to_json.jinja"),
    include_str!("../../../../dust_plugin_serde/src/templates/from_json_helper.jinja"),
    include_str!("../../../../dust_plugin_serde/src/templates/to_json_helper.jinja"),
);

/// Source inputs that affect generated HTTP client behavior.
const HTTP_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../../dust_http_client_plugin/src/lib.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/build.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/constants.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/model.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/util.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/parse/mod.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/parse/args.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/parse/http.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/validate/mod.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/validate/class.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/validate/endpoint.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/validate/param.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/validate/finalize.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/emit/mod.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/emit/class.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/emit/fixture.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/emit/path.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/emit/request.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/emit/response.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/emit/test_file.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/emit/test_support.rs"),
    include_str!("../../../../dust_http_client_plugin/src/plugin/emit/types.rs"),
);

/// Source inputs that affect generated Flutter route behavior.
const ROUTE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../../dust_route_plugin/src/lib.rs"),
    include_str!("../../../../dust_route_plugin/src/plugin.rs"),
    include_str!("../../../../dust_route_plugin/src/plugin/analysis.rs"),
    include_str!("../../../../dust_route_plugin/src/plugin/constants.rs"),
    include_str!("../../../../dust_route_plugin/src/plugin/model.rs"),
    include_str!("../../../../dust_route_plugin/src/plugin/parse.rs"),
    include_str!("../../../../dust_route_plugin/src/plugin/validate/mod.rs"),
    include_str!("../../../../dust_route_plugin/src/plugin/emit/mod.rs"),
);

/// Source inputs that affect generated Flutter state behavior.
const STATE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../../dust_state_plugin/src/lib.rs"),
    include_str!("../../../../dust_state_plugin/src/plugin.rs"),
    include_str!("../../../../dust_state_plugin/src/plugin/analysis.rs"),
    include_str!("../../../../dust_state_plugin/src/plugin/constants.rs"),
    include_str!("../../../../dust_state_plugin/src/plugin/emit.rs"),
    include_str!("../../../../dust_state_plugin/src/plugin/model.rs"),
    include_str!("../../../../dust_state_plugin/src/plugin/parse.rs"),
    include_str!("../../../../dust_state_plugin/src/plugin/validate.rs"),
);

/// Source inputs that affect generated DB behavior.
const DB_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../../dust_db_plugin/src/lib.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/mod.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/constants.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/model.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/parse.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/emit.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/validate.rs"),
);

/// Hashes UTF-8 text using Dust's stable cache hash algorithm.
pub(crate) fn hash_text(text: &str) -> u64 {
    hash_bytes(text.as_bytes())
}

/// Hashes raw bytes using FNV-1a.
fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 1469598103934665603_u64;
    update_hash_bytes(&mut hash, bytes);
    hash
}

/// Mixes bytes into an existing FNV-1a hash state.
fn update_hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = (*hash).wrapping_mul(1099511628211);
    }
}
/// Computes the codegen tool hash for the selected plugin registry mode.
pub(crate) fn codegen_tool_hash_for_selection(selection: RegistrySelection) -> CodegenToolHash {
    let mut combined = String::new();
    combined.push_str(selection.cache_salt());
    combined.push('\0');
    combined.push_str(CODEGEN_CORE_FINGERPRINT_INPUT);
    combined.push_str(DERIVE_PLUGIN_FINGERPRINT_INPUT);
    combined.push_str(SERDE_PLUGIN_FINGERPRINT_INPUT);
    combined.push_str(HTTP_PLUGIN_FINGERPRINT_INPUT);
    combined.push_str(ROUTE_PLUGIN_FINGERPRINT_INPUT);
    combined.push_str(STATE_PLUGIN_FINGERPRINT_INPUT);
    combined.push_str(DB_PLUGIN_FINGERPRINT_INPUT);

    CodegenToolHash {
        hash: hash_text(&combined),
    }
}
