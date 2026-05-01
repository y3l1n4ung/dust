mod apply;
mod batch;
mod process;
mod support;
mod work;

use std::time::Instant;

use crate::{
    context::CachedDriverContext,
    progress::{ProgressEvent, ProgressPhase},
    request::BuildRequest,
    result::CommandResult,
};

pub(crate) use apply::{ApplyOutcomeConfig, apply_indexed_outcomes, flush_cache_into_result};
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

    apply_indexed_outcomes(
        indexed,
        ApplyOutcomeConfig {
            cache_root: &workspace.cache_root,
            package_config_hash,
            tool_hash,
            fail_fast: request.fail_fast,
        },
        &mut cache,
        &mut result,
        None,
    );
    flush_cache_into_result(&cache, &mut result);
    result.cache = Some(cache_report);
    result.elapsed_ms = started.elapsed().as_millis();
    result
}
