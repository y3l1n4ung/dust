use std::time::Instant;

use dust_cache::{CacheEntry, WorkspaceCache};
use dust_diagnostics::Diagnostic;
use dust_workspace::discover_workspace;

use crate::{
    build::{
        codegen_tool_hash, default_registry, prepare_and_process_batch, read_package_config_hash,
    },
    catalog::build_symbol_catalog,
    progress::ProgressPhase,
    request::CheckRequest,
    result::{CacheReport, CheckedLibrary, CommandResult},
};

/// Runs a no-write freshness check across the discovered workspace.
pub fn run_check(request: CheckRequest) -> CommandResult {
    let started = Instant::now();
    let mut result = CommandResult::default();

    let workspace = match discover_workspace(&request.cwd) {
        Ok(workspace) => workspace,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };

    let registry = default_registry();
    let catalog = build_symbol_catalog(&registry);
    let tool_hash = codegen_tool_hash();
    let package_config_hash = match read_package_config_hash(&workspace.package_config.path) {
        Ok(hash) => hash,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };
    let mut cache = match WorkspaceCache::load(&workspace.root) {
        Ok(cache) => cache,
        Err(error) => {
            result.diagnostics.push(Diagnostic::error(format!(
                "failed to load Dust cache `{}`: {error}",
                workspace
                    .root
                    .join(".dart_tool/dust/build_cache_v1.json")
                    .display()
            )));
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };
    let mut cache_report = CacheReport {
        path: cache.path().to_path_buf(),
        ..CacheReport::default()
    };

    let indexed = prepare_and_process_batch(
        &workspace.root,
        &workspace.libraries,
        package_config_hash,
        tool_hash,
        &cache,
        &catalog,
        &registry,
        false,
        request.fail_fast,
        request.jobs,
        1,
        ProgressPhase::Build,
        None,
        &mut cache_report,
    );

    for indexed_outcome in indexed {
        let has_error = indexed_outcome
            .outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error());
        if let Some(expected_output_hash) = indexed_outcome.outcome.expected_output_hash {
            if let Some(source_hash) = indexed_outcome.source_hash {
                cache.insert(
                    &workspace.root,
                    &indexed_outcome.library.source_path,
                    CacheEntry {
                        source_hash,
                        package_config_hash,
                        tool_hash,
                        expected_output_hash,
                    },
                );
            }
        } else {
            cache.remove(&workspace.root, &indexed_outcome.library.source_path);
        }

        result.checked_libraries.push(CheckedLibrary {
            source_path: indexed_outcome.outcome.artifact.source_path.clone(),
            output_path: indexed_outcome.outcome.artifact.output_path.clone(),
            stale: !has_error && indexed_outcome.outcome.artifact.changed,
            cached: indexed_outcome.outcome.artifact.cached,
        });
        result
            .diagnostics
            .extend(indexed_outcome.outcome.diagnostics);

        if request.fail_fast && has_error {
            break;
        }
    }

    if let Err(error) = cache.flush() {
        result.diagnostics.push(Diagnostic::error(format!(
            "failed to persist Dust cache `{}`: {error}",
            cache.path().display()
        )));
    }
    result.cache = Some(cache_report);
    result.elapsed_ms = started.elapsed().as_millis();
    result
}
