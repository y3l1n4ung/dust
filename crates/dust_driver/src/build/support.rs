use std::{fs, io, path::Path, sync::Arc};

use dust_diagnostics::Diagnostic;
use dust_http_client_plugin::register_plugin as register_http_client_plugin;
use dust_plugin_api::PluginRegistry;
use dust_plugin_derive::register_plugin as register_derive_plugin;
use dust_plugin_serde::register_plugin as register_serde_plugin;
use dust_route_plugin::register_plugin as register_route_plugin;
use dust_state_plugin::register_plugin as register_state_plugin;
use dust_workspace::SourceLibrary;

use crate::build::process::LoadedLibraryInput;

const CODEGEN_FINGERPRINT_INPUT: &str = concat!(
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
    include_str!("../../../dust_plugin_derive/src/analysis.rs"),
    include_str!("../../../dust_plugin_derive/src/plugin.rs"),
    include_str!("../../../dust_plugin_derive/src/features/debug.rs"),
    include_str!("../../../dust_plugin_derive/src/features/eq_hash.rs"),
    include_str!("../../../dust_plugin_derive/src/features/clone_copy_with.rs"),
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
    include_str!("../../../dust_route_plugin/src/lib.rs"),
    include_str!("../../../dust_state_plugin/src/lib.rs"),
    include_str!("../../../dust_route_plugin/src/plugin.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/analysis.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/constants.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/model.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/parse.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/validate/mod.rs"),
    include_str!("../../../dust_route_plugin/src/plugin/emit/mod.rs"),
    include_str!("../../../dust_state_plugin/src/plugin.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/analysis.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/constants.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/emit.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/model.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/parse.rs"),
    include_str!("../../../dust_state_plugin/src/plugin/validate.rs"),
    include_str!("../../../dust_emitter/src/emit.rs"),
    include_str!("../../../dust_emitter/src/merge.rs"),
    include_str!("../../../dust_emitter/src/write.rs"),
    include_str!("../../../dust_emitter/src/writer.rs"),
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
    tool_hash: u64,
) -> Result<LoadedLibraryInput, Diagnostic> {
    let source = fs::read_to_string(&library.source_path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read `{}`: {error}",
            library.source_path.display()
        ))
    })?;
    let source_hash = hash_text(&source);
    let checked_output_hash = cache_fingerprint
        .filter(|entry| {
            entry.source_hash == source_hash
                && entry.package_config_hash == package_config_hash
                && entry.tool_hash == tool_hash
        })
        .map(|entry| {
            read_optional_hashes(&entry.output_paths, entry.allow_missing_primary).map_err(
                |error| {
                    Diagnostic::error(format!(
                        "failed to read generated outputs for `{}`: {error}",
                        library.source_path.display()
                    ))
                },
            )
        })
        .transpose()?;

    Ok(LoadedLibraryInput {
        source_hash,
        source: Arc::<str>::from(source),
        checked_output_hash,
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
    let mut hash = 1469598103934665603_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

pub(crate) fn matches_cache_metadata(
    entry: &dust_cache::CacheEntry,
    input: &LoadedLibraryInput,
    package_config_hash: u64,
    tool_hash: u64,
) -> bool {
    entry.source_hash == input.source_hash
        && entry.package_config_hash == package_config_hash
        && entry.tool_hash == tool_hash
}

pub(crate) fn codegen_tool_hash() -> u64 {
    hash_text(CODEGEN_FINGERPRINT_INPUT)
}

pub(crate) fn default_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
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
}

pub(crate) fn hash_output_set<'a>(outputs: impl IntoIterator<Item = (&'a Path, &'a str)>) -> u64 {
    let mut combined = String::new();
    for (path, source) in outputs {
        combined.push_str(&path.to_string_lossy());
        combined.push('\0');
        combined.push_str(source);
        combined.push('\0');
    }
    hash_text(&combined)
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
