mod batch;
mod process;
mod support;

use std::time::Instant;

use dust_cache::{CacheEntry, WorkspaceCache};
use dust_diagnostics::Diagnostic;
use dust_workspace::discover_workspace;

use crate::{
    catalog::build_symbol_catalog,
    progress::{ProgressEvent, ProgressPhase},
    request::BuildRequest,
    result::{CacheReport, CommandResult},
};

pub(crate) use batch::BatchConfig;
pub(crate) use batch::prepare_and_process_batch;
pub(crate) use support::{
    codegen_tool_hash, default_registry, hash_text, read_package_config_hash,
};

/// Runs one writing build across the discovered workspace.
pub fn run_build(request: BuildRequest) -> CommandResult {
    run_build_inner(request, None)
}

/// Runs one writing build across the discovered workspace while emitting progress events.
pub fn run_build_with_progress<F>(request: BuildRequest, progress: F) -> CommandResult
where
    F: Fn(ProgressEvent) + Send + Sync,
{
    run_build_inner(request, Some(&progress))
}

fn run_build_inner(
    request: BuildRequest,
    progress: Option<&(dyn Fn(ProgressEvent) + Send + Sync + '_)>,
) -> CommandResult {
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
    let mut cache = match WorkspaceCache::load(&workspace.cache_root) {
        Ok(cache) => cache,
        Err(error) => {
            result.diagnostics.push(Diagnostic::error(format!(
                "failed to load Dust cache `{}`: {error}",
                workspace
                    .cache_root
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
        BatchConfig {
            cache_root: &workspace.cache_root,
            package_config_hash,
            tool_hash,
            cache: &cache,
            catalog: &catalog,
            registry: &registry,
            write_output: true,
            fail_fast: request.fail_fast,
            jobs: request.jobs,
            file_id_base: 1,
            phase: ProgressPhase::Build,
            progress,
        },
        &workspace.libraries,
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
                    &workspace.cache_root,
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
            cache.remove(&workspace.cache_root, &indexed_outcome.library.source_path);
        }
        result
            .diagnostics
            .extend(indexed_outcome.outcome.diagnostics);
        result
            .build_artifacts
            .push(indexed_outcome.outcome.artifact);

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
