use std::{
    thread,
    time::{Duration, Instant},
};

mod snapshot;

use dust_cache::{CacheEntry, WorkspaceCache};
use dust_diagnostics::Diagnostic;
use dust_workspace::discover_workspace;

use crate::{
    build::{
        BatchConfig, codegen_tool_hash, default_registry, prepare_and_process_batch,
        read_package_config_hash,
    },
    catalog::build_symbol_catalog,
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

    for indexed_outcome in initial {
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
                        analysis_snapshot: indexed_outcome.outcome.analysis_snapshot,
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
            if let Err(error) = cache.flush() {
                result.diagnostics.push(Diagnostic::error(format!(
                    "failed to persist Dust cache `{}`: {error}",
                    cache.path().display()
                )));
            }
            result.watch = Some(WatchReport {
                cycles: 0,
                rebuild_batches: 0,
                rebuilt_libraries: Vec::new(),
            });
            result.cache = Some(cache_report);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
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

        for indexed_outcome in rebuilt {
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
                            analysis_snapshot: indexed_outcome.outcome.analysis_snapshot,
                        },
                    );
                }
            } else {
                cache.remove(&workspace.cache_root, &indexed_outcome.library.source_path);
            }
            result
                .diagnostics
                .extend(indexed_outcome.outcome.diagnostics);
            watch
                .rebuilt_libraries
                .push(indexed_outcome.outcome.artifact.source_path.clone());
            result
                .build_artifacts
                .push(indexed_outcome.outcome.artifact);

            if request.fail_fast && has_error {
                if let Err(error) = cache.flush() {
                    result.diagnostics.push(Diagnostic::error(format!(
                        "failed to persist Dust cache `{}`: {error}",
                        cache.path().display()
                    )));
                }
                result.watch = Some(watch);
                result.cache = Some(cache_report);
                result.elapsed_ms = started.elapsed().as_millis();
                return result;
            }
        }
    }

    if let Err(error) = cache.flush() {
        result.diagnostics.push(Diagnostic::error(format!(
            "failed to persist Dust cache `{}`: {error}",
            cache.path().display()
        )));
    }
    result.watch = Some(watch);
    result.cache = Some(cache_report);
    result.elapsed_ms = started.elapsed().as_millis();
    result
}
