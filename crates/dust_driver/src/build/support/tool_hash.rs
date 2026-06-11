use super::registry::RegistrySelection;

#[derive(Clone, Copy)]
pub(crate) struct CodegenToolHash {
    hash: u64,
}

impl CodegenToolHash {
    pub(crate) fn value(self) -> u64 {
        self.hash
    }
}

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

const DERIVE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../../dust_plugin_derive/src/analysis.rs"),
    include_str!("../../../../dust_plugin_derive/src/plugin.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/debug.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/eq_hash.rs"),
    include_str!("../../../../dust_plugin_derive/src/features/clone_copy_with.rs"),
);

const SERDE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../../dust_plugin_serde/src/plugin.rs"),
    include_str!("../../../../dust_plugin_serde/src/validate.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit_class.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit_enum.rs"),
    include_str!("../../../../dust_plugin_serde/src/emit_support.rs"),
    include_str!("../../../../dust_plugin_serde/src/writer.rs"),
    include_str!("../../../../dust_plugin_serde/src/writer_expr.rs"),
    include_str!("../../../../dust_plugin_serde/src/writer_model.rs"),
    include_str!("../../../../dust_plugin_serde/src/writer_type.rs"),
);

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

const DB_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../../dust_db_plugin/src/lib.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/mod.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/constants.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/model.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/parse.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/emit.rs"),
    include_str!("../../../../dust_db_plugin/src/plugin/validate.rs"),
);

pub(crate) fn hash_text(text: &str) -> u64 {
    hash_bytes(text.as_bytes())
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 1469598103934665603_u64;
    update_hash_bytes(&mut hash, bytes);
    hash
}

fn update_hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = (*hash).wrapping_mul(1099511628211);
    }
}
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
