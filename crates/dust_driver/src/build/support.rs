use std::{fs, io, path::Path, sync::Arc};

use dust_diagnostics::Diagnostic;
use dust_plugin_api::PluginRegistry;
use dust_plugin_derive::register_plugin as register_derive_plugin;
use dust_plugin_serde::register_plugin as register_serde_plugin;
use dust_workspace::SourceLibrary;

use crate::build::process::LoadedLibraryInput;

const CODEGEN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("../build.rs"),
    include_str!("../check.rs"),
    include_str!("../watch.rs"),
    include_str!("../lower.rs"),
    include_str!("batch.rs"),
    include_str!("batch/load.rs"),
    include_str!("batch/execute.rs"),
    include_str!("process/mod.rs"),
    include_str!("process/scan.rs"),
    include_str!("support.rs"),
    include_str!("work.rs"),
    include_str!("../../../dust_plugin_api/src/analysis.rs"),
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
    include_str!("../../../dust_emitter/src/emit.rs"),
    include_str!("../../../dust_emitter/src/writer.rs"),
);

#[derive(Clone, Copy)]
pub(crate) struct CacheFingerprint {
    pub(crate) source_hash: u64,
    pub(crate) package_config_hash: u64,
    pub(crate) tool_hash: u64,
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
        .map(|_| {
            read_optional_hash(&library.output_path).map_err(|error| {
                Diagnostic::error(format!(
                    "failed to read `{}`: {error}",
                    library.output_path.display()
                ))
            })
        })
        .transpose()?;

    Ok(LoadedLibraryInput {
        source_hash,
        source: Arc::<str>::from(source),
        checked_output_hash,
    })
}

pub(crate) fn read_package_config_hash(path: &Path) -> Result<u64, Diagnostic> {
    let source = fs::read_to_string(path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read package configuration `{}`: {error}",
            path.display()
        ))
    })?;
    Ok(hash_text(&source))
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
}

fn read_optional_hash(path: &Path) -> io::Result<Option<u64>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(hash_text(&source))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
