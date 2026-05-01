use std::{
    thread,
    time::{Duration, Instant},
};

mod snapshot;

use dust_cache::WorkspaceCache;

use crate::{
    build::{
        ApplyOutcomeConfig, BatchConfig, apply_indexed_outcomes, flush_cache_into_result,
        prepare_and_process_batch, read_package_config_hash,
    },
    context::CachedDriverContext,
    progress::{ProgressEvent, ProgressPhase},
    request::WatchRequest,
    result::{CacheReport, CommandResult, WatchReport},
};

use self::snapshot::{build_snapshot, changed_libraries};

/// Runs initial build plus repeated poll-based rebuild detection.
pub fn run_watch(request: WatchRequest) -> CommandResult {
    run_watch_inner(request, None)
}

/// Runs initial build plus repeated poll-based rebuild detection while emitting progress events.
pub fn run_watch_with_progress<F>(request: WatchRequest, progress: F) -> CommandResult
where
    F: Fn(ProgressEvent) + Send + Sync,
{
    run_watch_inner(request, Some(&progress))
}

fn run_watch_inner(
    request: WatchRequest,
    progress: Option<&(dyn Fn(ProgressEvent) + Send + Sync + '_)>,
) -> CommandResult {
    let started = Instant::now();
    let mut result = CommandResult::default();

    let CachedDriverContext {
        workspace,
        registry,
        catalog,
        tool_hash,
        mut cache,
        mut cache_report,
        ..
    } = match CachedDriverContext::load(&request.cwd) {
        Ok(context) => context,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };
    let mut package_config_hash = match read_package_config_hash(&workspace.package_config.path) {
        Ok(hash) => hash,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };

    let initial = prepare_and_process_batch(
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
            phase: ProgressPhase::WatchInitial,
            progress,
        },
        &workspace.libraries,
        &mut cache_report,
    );

    if apply_indexed_outcomes(
        initial,
        ApplyOutcomeConfig {
            cache_root: &workspace.cache_root,
            package_config_hash,
            tool_hash,
            fail_fast: request.fail_fast,
        },
        &mut cache,
        &mut result,
        None,
    ) {
        return finish_watch_result(result, &cache, cache_report, empty_watch_report(), started);
    }

    let mut snapshot = match build_snapshot(&request.cwd) {
        Ok(snapshot) => snapshot,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.watch = Some(WatchReport {
                cycles: 0,
                rebuild_batches: 0,
                rebuilt_libraries: Vec::new(),
            });
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };

    let mut watch = WatchReport {
        cycles: 0,
        rebuild_batches: 0,
        rebuilt_libraries: Vec::new(),
    };
    let max_cycles = request.max_cycles.unwrap_or(u32::MAX);

    for cycle in 0..max_cycles {
        thread::sleep(Duration::from_millis(request.poll_interval_ms));
        watch.cycles = cycle + 1;

        let next_snapshot = match build_snapshot(&request.cwd) {
            Ok(snapshot) => snapshot,
            Err(diagnostic) => {
                result.diagnostics.push(diagnostic);
                break;
            }
        };

        let changed = changed_libraries(&snapshot, &next_snapshot);
        package_config_hash = next_snapshot
            .package_config_hash
            .unwrap_or(package_config_hash);
        snapshot = next_snapshot;

        if changed.is_empty() {
            continue;
        }

        watch.rebuild_batches += 1;
        let rebuilt = prepare_and_process_batch(
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
                file_id_base: 10_000,
                phase: ProgressPhase::WatchRebuild,
                progress,
            },
            &changed,
            &mut cache_report,
        );

        if apply_indexed_outcomes(
            rebuilt,
            ApplyOutcomeConfig {
                cache_root: &workspace.cache_root,
                package_config_hash,
                tool_hash,
                fail_fast: request.fail_fast,
            },
            &mut cache,
            &mut result,
            Some(&mut watch.rebuilt_libraries),
        ) {
            return finish_watch_result(result, &cache, cache_report, watch, started);
        }
    }

    finish_watch_result(result, &cache, cache_report, watch, started)
}

fn empty_watch_report() -> WatchReport {
    WatchReport {
        cycles: 0,
        rebuild_batches: 0,
        rebuilt_libraries: Vec::new(),
    }
}

fn finish_watch_result(
    mut result: CommandResult,
    cache: &WorkspaceCache,
    cache_report: CacheReport,
    watch: WatchReport,
    started: Instant,
) -> CommandResult {
    flush_cache_into_result(cache, &mut result);
    result.watch = Some(watch);
    result.cache = Some(cache_report);
    result.elapsed_ms = started.elapsed().as_millis();
    result
}
