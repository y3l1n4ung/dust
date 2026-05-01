use std::path::{Path, PathBuf};

use dust_cache::{CacheEntry, WorkspaceCache};
use dust_diagnostics::Diagnostic;

use crate::{
    build::process::{BuildOutcome, IndexedBuildOutcome},
    result::CommandResult,
};

pub(crate) struct ApplyOutcomeConfig<'a> {
    pub(crate) cache_root: &'a Path,
    pub(crate) package_config_hash: u64,
    pub(crate) tool_hash: u64,
    pub(crate) fail_fast: bool,
}

pub(crate) fn apply_indexed_outcomes(
    indexed: Vec<IndexedBuildOutcome>,
    config: ApplyOutcomeConfig<'_>,
    cache: &mut WorkspaceCache,
    result: &mut CommandResult,
    mut rebuilt_libraries: Option<&mut Vec<PathBuf>>,
) -> bool {
    for indexed_outcome in indexed {
        let IndexedBuildOutcome {
            library,
            source_hash,
            outcome,
            ..
        } = indexed_outcome;
        let BuildOutcome {
            diagnostics,
            artifact,
            expected_output_hash,
            analysis_snapshot,
        } = outcome;
        let has_error = diagnostics.iter().any(|diagnostic| diagnostic.is_error());

        if let Some(expected_output_hash) = expected_output_hash {
            if let Some(source_hash) = source_hash {
                cache.insert(
                    config.cache_root,
                    &library.source_path,
                    CacheEntry {
                        source_hash,
                        package_config_hash: config.package_config_hash,
                        tool_hash: config.tool_hash,
                        expected_output_hash,
                        analysis_snapshot,
                    },
                );
            }
        } else {
            cache.remove(config.cache_root, &library.source_path);
        }

        result.diagnostics.extend(diagnostics);
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

pub(crate) fn flush_cache_into_result(cache: &WorkspaceCache, result: &mut CommandResult) {
    if let Err(error) = cache.flush() {
        result.diagnostics.push(Diagnostic::error(format!(
            "failed to persist Dust cache `{}`: {error}",
            cache.path().display()
        )));
    }
}
