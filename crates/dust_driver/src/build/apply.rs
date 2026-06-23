use std::path::{Path, PathBuf};

use dust_cache::{CacheEntry, WorkspaceCache};
use dust_diagnostics::Diagnostic;

use crate::{
    build::process::{BuildOutcome, IndexedBuildOutcome},
    result::CommandResult,
};

/// Cache and command settings used while applying per-library outcomes.
#[derive(Clone, Copy)]
pub(crate) struct ApplyOutcomeConfig<'a> {
    /// Root directory for persisted Dust cache entries.
    pub(crate) cache_root: &'a Path,
    /// Hash of package and Dust configuration files.
    pub(crate) package_config_hash: u64,
    /// Whether applying should stop after the first error outcome.
    pub(crate) fail_fast: bool,
}

/// Applies processed library outcomes to cache state and the command result.
pub(crate) fn apply_indexed_outcomes(
    indexed: impl IntoIterator<Item = IndexedBuildOutcome>,
    config: ApplyOutcomeConfig<'_>,
    cache: &mut WorkspaceCache,
    result: &mut CommandResult,
    mut rebuilt_libraries: Option<&mut Vec<PathBuf>>,
) -> bool {
    for indexed_outcome in indexed {
        let IndexedBuildOutcome {
            library,
            source_hash,
            tool_hash,
            outcome,
            ..
        } = indexed_outcome;
        let BuildOutcome {
            diagnostics,
            diagnostic_file,
            artifact,
            expected_output_hash,
            analysis_snapshot,
        } = outcome;
        let has_error = diagnostics.iter().any(|diagnostic| diagnostic.is_error());
        let has_labels = diagnostics.iter().any(Diagnostic::has_labels);

        if let Some(expected_output_hash) = expected_output_hash {
            if let (Some(source_hash), Some(tool_hash)) = (source_hash, tool_hash) {
                cache.insert(
                    config.cache_root,
                    &library.source_path,
                    CacheEntry {
                        source_hash,
                        package_config_hash: config.package_config_hash,
                        tool_hash,
                        expected_output_hash,
                        auxiliary_output_paths: artifact.auxiliary_output_paths.clone(),
                        analysis_snapshot,
                    },
                );
            }
        } else {
            cache.remove(config.cache_root, &library.source_path);
        }

        result.diagnostics.extend(diagnostics);
        if has_labels {
            if let Some(diagnostic_file) = diagnostic_file {
                result.diagnostic_files.push(diagnostic_file);
            }
        }
        if let Some(paths) = rebuilt_libraries.as_deref_mut() {
            paths.push(artifact.source_path.clone());
        }
        result.build_artifacts.push(artifact);

        if config.fail_fast && has_error {
            return true;
        }
    }

    false
}

/// Persists cache metadata and reports a diagnostic if flushing fails.
pub(crate) fn flush_cache_into_result(cache: &mut WorkspaceCache, result: &mut CommandResult) {
    if let Err(error) = cache.flush() {
        result.diagnostics.push(Diagnostic::error(format!(
            "failed to persist Dust cache `{}`: {error}",
            cache.path().display()
        )));
    }
}
