use std::{fs, io, path::Path, sync::Arc};

use dust_diagnostics::Diagnostic;
use dust_emitter::hash_output_set;
use dust_workspace::SourceLibrary;

use crate::build::process::LoadedLibraryInput;

use super::tool_hash::{CodegenToolHash, hash_text};

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
