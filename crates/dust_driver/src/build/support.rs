use std::{fs, io, path::Path, sync::Arc};

use dust_db_plugin::{
    register_plugin_with_options as register_db_plugin_with_options,
    register_row_plugin as register_db_row_plugin,
};
use dust_diagnostics::Diagnostic;
use dust_emitter::hash_output_set;
use dust_http_client_plugin::register_plugin as register_http_client_plugin;
use dust_plugin_api::{DustPlugin, PluginContribution, PluginRegistry, SymbolPlan};
use dust_plugin_derive::register_plugin as register_derive_plugin;
use dust_plugin_serde::register_plugin as register_serde_plugin;
use dust_route_plugin::register_plugin as register_route_plugin;
use dust_state_plugin::register_plugin as register_state_plugin;
use dust_workspace::SourceLibrary;

use crate::build::process::LoadedLibraryInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegistrySelection {
    All,
    DbOnly { offline: bool, write_metadata: bool },
}

impl RegistrySelection {
    pub(crate) fn for_build(value: crate::request::DbRequestOptions) -> Self {
        Self::from_db_request(value, true)
    }

    pub(crate) fn for_check(value: crate::request::DbRequestOptions) -> Self {
        Self::from_db_request(value, false)
    }

    fn from_db_request(value: crate::request::DbRequestOptions, write_metadata: bool) -> Self {
        if value.only_db {
            return Self::DbOnly {
                offline: value.offline,
                write_metadata,
            };
        }
        Self::All
    }

    pub(crate) fn cache_salt(self) -> &'static str {
        match self {
            Self::All => "registry:all",
            Self::DbOnly {
                offline: false,
                write_metadata: true,
            } => "registry:db-only:online:write-metadata",
            Self::DbOnly {
                offline: false,
                write_metadata: false,
            } => "registry:db-only:online:no-metadata",
            Self::DbOnly {
                offline: true,
                write_metadata: true,
            } => "registry:db-only:offline:write-metadata",
            Self::DbOnly {
                offline: true,
                write_metadata: false,
            } => "registry:db-only:offline:no-metadata",
        }
    }
}

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
    include_str!("../build.rs"),
    include_str!("../check.rs"),
    include_str!("../context.rs"),
    include_str!("../watch.rs"),
    include_str!("../lower.rs"),
    include_str!("../lower/inheritance.rs"),
    include_str!("../lower/parse_support.rs"),
    include_str!("../lower/serde.rs"),
    include_str!("../lower/serde_parse.rs"),
    include_str!("../lower/type_parse.rs"),
    include_str!("apply.rs"),
    include_str!("batch.rs"),
    include_str!("batch/load.rs"),
    include_str!("batch/execute.rs"),
    include_str!("process/mod.rs"),
    include_str!("process/execute.rs"),
    include_str!("process/output.rs"),
    include_str!("process/scan.rs"),
    include_str!("support.rs"),
    include_str!("work.rs"),
    include_str!("../../../dust_plugin_api/src/analysis.rs"),
    include_str!("../../../dust_plugin_api/src/contribution.rs"),
    include_str!("../../../dust_plugin_api/src/plugin.rs"),
    include_str!("../../../dust_plugin_api/src/registry.rs"),
    include_str!("../../../dust_plugin_api/src/symbols.rs"),
    include_str!("../../../dust_dart_emit/src/lib.rs"),
    include_str!("../../../dust_dart_emit/src/rename.rs"),
    include_str!("../../../dust_dart_emit/src/type_render.rs"),
    include_str!("../../../dust_workspace/src/config.rs"),
    include_str!("../../../dust_workspace/src/discover.rs"),
    include_str!("../../../dust_workspace/src/output_policy.rs"),
    include_str!("../../../dust_workspace/src/package_config.rs"),
    include_str!("../../../dust_workspace/src/pubspec.rs"),
    include_str!("../../../dust_workspace/src/root.rs"),
    include_str!("../../../dust_workspace/src/workspace.rs"),
    include_str!("../../../dust_emitter/src/emit.rs"),
    include_str!("../../../dust_emitter/src/merge.rs"),
    include_str!("../../../dust_emitter/src/write.rs"),
    include_str!("../../../dust_emitter/src/writer.rs"),
);

const DERIVE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../dust_plugin_derive/src/analysis.rs"),
    include_str!("../../../dust_plugin_derive/src/plugin.rs"),
    include_str!("../../../dust_plugin_derive/src/features/debug.rs"),
    include_str!("../../../dust_plugin_derive/src/features/eq_hash.rs"),
    include_str!("../../../dust_plugin_derive/src/features/clone_copy_with.rs"),
);

const SERDE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../dust_plugin_serde/src/plugin.rs"),
    include_str!("../../../dust_plugin_serde/src/validate.rs"),
    include_str!("../../../dust_plugin_serde/src/emit.rs"),
    include_str!("../../../dust_plugin_serde/src/emit_class.rs"),
    include_str!("../../../dust_plugin_serde/src/emit_enum.rs"),
    include_str!("../../../dust_plugin_serde/src/emit_support.rs"),
    include_str!("../../../dust_plugin_serde/src/writer.rs"),
    include_str!("../../../dust_plugin_serde/src/writer_expr.rs"),
    include_str!("../../../dust_plugin_serde/src/writer_model.rs"),
    include_str!("../../../dust_plugin_serde/src/writer_type.rs"),
);

const HTTP_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../dust_http_client_plugin/src/lib.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/build.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/constants.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/model.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/util.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/parse/mod.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/parse/args.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/parse/http.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/validate/mod.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/validate/class.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/validate/endpoint.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/validate/param.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/validate/finalize.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/emit/mod.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/emit/class.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/emit/fixture.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/emit/path.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/emit/request.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/emit/response.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/emit/test_file.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/emit/test_support.rs"),
    include_str!("../../../dust_http_client_plugin/src/plugin/emit/types.rs"),
);

const ROUTE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../dust_route_plugin/src/lib.rs"),
    include_str!("../../../dust_route_plugin/src/plugin.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/analysis.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/constants.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/model.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/parse.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/validate/mod.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/emit/mod.rs"),
);

const STATE_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../dust_state_plugin/src/lib.rs"),
    include_str!("../../../dust_state_plugin/src/plugin.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/analysis.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/constants.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/emit.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/model.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/parse.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/validate.rs"),
);

const DB_PLUGIN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../../../dust_db_plugin/src/lib.rs"),
    include_str!("../../../dust_db_plugin/src/plugin/mod.rs"),
    include_str!("../../../dust_db_plugin/src/plugin/constants.rs"),
    include_str!("../../../dust_db_plugin/src/plugin/model.rs"),
    include_str!("../../../dust_db_plugin/src/plugin/parse.rs"),
    include_str!("../../../dust_db_plugin/src/plugin/emit.rs"),
    include_str!("../../../dust_db_plugin/src/plugin/validate.rs"),
);

#[derive(Clone)]
pub(crate) struct CacheFingerprint {
    pub(crate) source_hash: u64,
    pub(crate) package_config_hash: u64,
    pub(crate) tool_hash: u64,
    pub(crate) output_paths: Vec<std::path::PathBuf>,
    pub(crate) allow_missing_primary: bool,
}

pub(crate) fn load_library_input(
    library: &SourceLibrary,
    cache_fingerprint: Option<CacheFingerprint>,
    package_config_hash: u64,
    tool_hash: CodegenToolHash,
) -> Result<LoadedLibraryInput, Diagnostic> {
    let source = fs::read_to_string(&library.source_path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read `{}`: {error}",
            library.source_path.display()
        ))
    })?;
    let source_hash = hash_text(&source);
    let tool_hash = tool_hash.value();
    let previous_output_hash = cache_fingerprint
        .as_ref()
        .filter(|entry| {
            entry.source_hash == source_hash && entry.package_config_hash == package_config_hash
        })
        .map(|entry| cached_output_hash(entry, &library.source_path))
        .transpose()?;
    let checked_output_hash = cache_fingerprint
        .as_ref()
        .filter(|entry| entry.tool_hash == tool_hash)
        .and(previous_output_hash);

    Ok(LoadedLibraryInput {
        source_hash,
        tool_hash,
        source: Arc::<str>::from(source),
        checked_output_hash,
        previous_output_hash,
    })
}

pub(crate) fn read_workspace_config_hash(
    package_config_path: &Path,
    dust_config_path: Option<&Path>,
) -> Result<u64, Diagnostic> {
    let package_config = fs::read_to_string(package_config_path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read package configuration `{}`: {error}",
            package_config_path.display()
        ))
    })?;
    let dust_config = match dust_config_path {
        Some(path) => Some(fs::read_to_string(path).map_err(|error| {
            Diagnostic::error(format!(
                "failed to read Dust configuration `{}`: {error}",
                path.display()
            ))
        })?),
        None => None,
    };

    let mut combined = String::new();
    combined.push_str(&package_config);
    combined.push('\0');
    if let Some(dust_config) = dust_config {
        combined.push_str(&dust_config);
    }
    Ok(hash_text(&combined))
}

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

pub(crate) fn matches_cache_metadata(
    entry: &dust_cache::CacheEntry,
    input: &LoadedLibraryInput,
    package_config_hash: u64,
) -> bool {
    entry.source_hash == input.source_hash
        && entry.package_config_hash == package_config_hash
        && entry.tool_hash == input.tool_hash
}

pub(crate) fn route_only_analysis(snapshot: &dust_plugin_api::LibraryAnalysisSnapshot) -> bool {
    snapshot.string_set("dust_route.routes.v1").is_some()
        && snapshot.string_set("dust_route.routers.v1").is_none()
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

pub(crate) fn default_registry() -> PluginRegistry {
    registry_for_selection(RegistrySelection::All)
}

pub(crate) fn registry_for_selection(selection: RegistrySelection) -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    match selection {
        RegistrySelection::All => {}
        RegistrySelection::DbOnly {
            offline,
            write_metadata,
        } => {
            registry
                .register(Box::new(DbModePassThroughPlugin))
                .expect("db pass-through plugin symbol ownership must be valid");
            registry
                .register(Box::new(register_db_plugin_with_options(
                    offline,
                    write_metadata,
                )))
                .expect("db plugin symbol ownership must be valid");
            return registry;
        }
    }

    registry
        .register(Box::new(register_derive_plugin()))
        .expect("derive plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_serde_plugin()))
        .expect("serde plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_http_client_plugin()))
        .expect("http client plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_route_plugin()))
        .expect("route plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_state_plugin()))
        .expect("state plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_db_row_plugin()))
        .expect("db plugin symbol ownership must be valid");
    registry
}

struct DbModePassThroughPlugin;

const DB_MODE_PASS_THROUGH_TRAITS: &[&str] = &[
    "derive_annotation::ToString",
    "derive_annotation::Debug",
    "derive_annotation::Eq",
    "derive_annotation::CopyWith",
    "derive_serde_annotation::Serialize",
    "derive_serde_annotation::Deserialize",
];

const DB_MODE_PASS_THROUGH_CONFIGS: &[&str] = &[
    "derive_serde_annotation::SerDe",
    "dust_http_client_annotation::HttpClient",
    "dust_http_client_annotation::GenerateTest",
    "dust_http_client_annotation::GET",
    "dust_http_client_annotation::POST",
    "dust_http_client_annotation::PUT",
    "dust_http_client_annotation::PATCH",
    "dust_http_client_annotation::DELETE",
    "dust_http_client_annotation::HEAD",
    "dust_http_client_annotation::OPTIONS",
    "dust_http_client_annotation::Path",
    "dust_http_client_annotation::Query",
    "dust_http_client_annotation::Queries",
    "dust_http_client_annotation::Header",
    "dust_http_client_annotation::Headers",
    "dust_http_client_annotation::HeaderMap",
    "dust_http_client_annotation::Body",
    "dust_http_client_annotation::Field",
    "dust_http_client_annotation::Part",
    "dust_http_client_annotation::Extra",
    "dust_http_client_annotation::FormUrlEncoded",
    "dust_http_client_annotation::MultiPart",
    "dust_http_client_annotation::HttpParse",
    "dust_router::Router",
    "dust_router::Route",
    "dust_router::GeneratedRoute",
    "dust_state::ViewModel",
];

impl DustPlugin for DbModePassThroughPlugin {
    fn plugin_name(&self) -> &'static str {
        "DbModePassThrough"
    }

    fn claimed_traits(&self) -> &'static [&'static str] {
        DB_MODE_PASS_THROUGH_TRAITS
    }

    fn claimed_configs(&self) -> &'static [&'static str] {
        DB_MODE_PASS_THROUGH_CONFIGS
    }

    fn validate(&self, _library: &dust_ir::LibraryIr) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn emit(&self, _library: &dust_ir::LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        PluginContribution::default()
    }
}

fn cached_output_hash(
    entry: &CacheFingerprint,
    source_path: &Path,
) -> Result<Option<u64>, Diagnostic> {
    read_optional_hashes(&entry.output_paths, entry.allow_missing_primary).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read generated outputs for `{}`: {error}",
            source_path.display()
        ))
    })
}

fn read_optional_hashes(
    paths: &[std::path::PathBuf],
    allow_missing_primary: bool,
) -> io::Result<Option<u64>> {
    let mut sources = Vec::with_capacity(paths.len());
    for (index, path) in paths.iter().enumerate() {
        let Some(source) = read_previous_output(path)? else {
            if allow_missing_primary && index == 0 {
                sources.push((path.as_path(), String::new()));
                continue;
            }
            return Ok(None);
        };
        sources.push((path.as_path(), source));
    }
    Ok(Some(hash_output_set(
        sources
            .iter()
            .map(|(path, source)| (*path, source.as_str())),
    )))
}

fn read_previous_output(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
