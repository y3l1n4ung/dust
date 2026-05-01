use std::time::Instant;

use crate::{
    build::{ApplyOutcomeConfig, BatchConfig, flush_cache_into_result, prepare_and_process_batch},
    context::CachedDriverContext,
    progress::ProgressPhase,
    request::CheckRequest,
    result::{CheckedLibrary, CommandResult},
};

/// Runs a no-write freshness check across the discovered workspace.
pub fn run_check(request: CheckRequest) -> CommandResult {
    let started = Instant::now();
    let mut result = CommandResult::default();

    let CachedDriverContext {
        workspace,
        registry,
        catalog,
        tool_hash,
        package_config_hash,
        mut cache,
        mut cache_report,
    } = match CachedDriverContext::load(&request.cwd) {
        Ok(context) => context,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };

    let indexed = prepare_and_process_batch(
        BatchConfig {
            cache_root: &workspace.cache_root,
            package_config_hash,
            tool_hash,
            cache: &cache,
            catalog: &catalog,
            registry: &registry,
            write_output: false,
            fail_fast: request.fail_fast,
            jobs: request.jobs,
            file_id_base: 1,
            phase: ProgressPhase::Build,
            progress: None,
        },
        &workspace.libraries,
        &mut cache_report,
    );

    let apply_config = ApplyOutcomeConfig {
        cache_root: &workspace.cache_root,
        package_config_hash,
        tool_hash,
        fail_fast: request.fail_fast,
    };

    for indexed_outcome in indexed {
        let has_error = indexed_outcome
            .outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error());
        result.checked_libraries.push(CheckedLibrary {
            source_path: indexed_outcome.outcome.artifact.source_path.clone(),
            output_path: indexed_outcome.outcome.artifact.output_path.clone(),
            stale: !has_error && indexed_outcome.outcome.artifact.changed,
            cached: indexed_outcome.outcome.artifact.cached,
        });

        if crate::build::apply_indexed_outcomes(
            std::iter::once(indexed_outcome),
            apply_config,
            &mut cache,
            &mut result,
            None,
        ) {
            break;
        }
    }

    flush_cache_into_result(&cache, &mut result);
    result.cache = Some(cache_report);
    result.elapsed_ms = started.elapsed().as_millis();
    result
}
