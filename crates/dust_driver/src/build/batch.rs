mod outcome;
mod progress;

use std::{
    path::Path,
    sync::{Arc, atomic::AtomicUsize},
    thread,
};

use dust_cache::WorkspaceCache;
use dust_text::FileId;
use dust_workspace::SourceLibrary;

use crate::{
    build::{
        process::{
            IndexedBuildOutcome, PendingLibrary, ProcessingConfig, collect_workspace_analysis,
            process_pending_library,
        },
        support::{CacheFingerprint, load_library_input, matches_cache_metadata},
    },
    progress::{ProgressEvent, ProgressPhase},
    result::CacheReport,
};

use self::outcome::{build_cached_outcome, build_load_error};
pub(crate) use self::progress::{ProgressReporter, ProgressSnapshot};

pub(crate) type ProgressCallback<'a> = dyn Fn(ProgressEvent) + Send + Sync + 'a;

#[derive(Clone, Copy)]
pub(crate) struct BatchConfig<'a> {
    pub(crate) cache_root: &'a Path,
    pub(crate) package_config_hash: u64,
    pub(crate) tool_hash: u64,
    pub(crate) cache: &'a WorkspaceCache,
    pub(crate) catalog: &'a dust_resolver::SymbolCatalog,
    pub(crate) registry: &'a dust_plugin_api::PluginRegistry,
    pub(crate) write_output: bool,
    pub(crate) fail_fast: bool,
    pub(crate) jobs: Option<usize>,
    pub(crate) file_id_base: u32,
    pub(crate) phase: ProgressPhase,
    pub(crate) progress: Option<&'a ProgressCallback<'a>>,
}

pub(crate) fn prepare_and_process_batch(
    config: BatchConfig<'_>,
    libraries: &[SourceLibrary],
    cache_report: &mut CacheReport,
) -> Vec<IndexedBuildOutcome> {
    let completed = Arc::new(AtomicUsize::new(0));
    let reporter =
        ProgressReporter::new(config.progress, &completed, config.phase, libraries.len());
    reporter.started_batch();

    let mut outcomes = Vec::new();

    let threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(libraries.len())
        .max(1);

    let mut groups = (0..threads).map(|_| Vec::new()).collect::<Vec<_>>();
    for (index, item) in libraries.iter().enumerate() {
        groups[index % threads].push((index, item));
    }

    let loaded_results = thread::scope(|scope| {
        let mut handles = Vec::new();
        for group in groups {
            handles.push(scope.spawn(move || {
                group
                    .into_iter()
                    .map(|(index, library)| {
                        let cache_fingerprint = config
                            .cache
                            .get(config.cache_root, &library.source_path)
                            .map(|entry| CacheFingerprint {
                                source_hash: entry.source_hash,
                                package_config_hash: entry.package_config_hash,
                                tool_hash: entry.tool_hash,
                            });
                        let input = load_library_input(
                            library,
                            cache_fingerprint,
                            config.package_config_hash,
                            config.tool_hash,
                        );
                        (index, library.clone(), input)
                    })
                    .collect::<Vec<_>>()
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.extend(handle.join().expect("load thread must not panic"));
        }
        results.sort_by_key(|(index, _, _)| *index);
        results
    });

    let mut workspace_analysis = dust_plugin_api::WorkspaceAnalysisBuilder::default();
    let mut pending = Vec::new();

    for (index, library, input_result) in loaded_results {
        let input = match input_result {
            Ok(input) => input,
            Err(diagnostic) => {
                outcomes.push(build_load_error(index, &library, diagnostic));
                reporter.finish(ProgressSnapshot {
                    library: &library,
                    cached: false,
                    written: false,
                    changed: false,
                    had_errors: true,
                    elapsed_ms: 0,
                });
                continue;
            }
        };

        if let Some(entry) = config.cache.get(config.cache_root, &library.source_path) {
            if matches_cache_metadata(entry, &input, config.package_config_hash, config.tool_hash) {
                if input.checked_output_hash != Some(Some(entry.expected_output_hash)) {
                    cache_report.misses += 1;
                    pending.push(PendingLibrary {
                        index,
                        file_id: FileId::new(config.file_id_base + index as u32),
                        library,
                        input,
                        pre_parsed: None,
                        analysis_snapshot: dust_plugin_api::LibraryAnalysisSnapshot::default(),
                    });
                    continue;
                }
                cache_report.hits += 1;
                workspace_analysis.merge_snapshot(&entry.analysis_snapshot);
                outcomes.push(build_cached_outcome(
                    index,
                    &library,
                    entry.expected_output_hash,
                    entry.analysis_snapshot.clone(),
                ));
                reporter.finish(ProgressSnapshot {
                    library: &library,
                    cached: true,
                    written: false,
                    changed: false,
                    had_errors: false,
                    elapsed_ms: 0,
                });
                continue;
            }
        }

        cache_report.misses += 1;
        pending.push(PendingLibrary {
            index,
            file_id: FileId::new(config.file_id_base + index as u32),
            library,
            input,
            pre_parsed: None,
            analysis_snapshot: dust_plugin_api::LibraryAnalysisSnapshot::default(),
        });
    }

    let (pending_analysis, pre_parsed_libraries, analysis_snapshots) =
        collect_workspace_analysis(&pending, config.registry);
    workspace_analysis.merge(pending_analysis);
    for ((pending, pre_parsed), analysis_snapshot) in pending
        .iter_mut()
        .zip(pre_parsed_libraries)
        .zip(analysis_snapshots)
    {
        pending.pre_parsed = pre_parsed;
        pending.analysis_snapshot = analysis_snapshot;
    }

    let workspace_analysis = Arc::new(workspace_analysis.build());

    let processing = ProcessingConfig {
        catalog: config.catalog,
        registry: config.registry,
        workspace_analysis: &workspace_analysis,
        write_output: config.write_output,
    };

    let desired_jobs = config.jobs.unwrap_or_else(default_parallel_jobs).max(1);
    let mut processed = if config.fail_fast || desired_jobs <= 1 || pending.len() <= 1 {
        process_pending_serial(pending, config.fail_fast, &processing, &reporter)
    } else {
        process_pending_parallel(pending, desired_jobs, &processing, &reporter)
    };

    outcomes.append(&mut processed);
    outcomes.sort_by_key(|outcome| outcome.index);
    outcomes
}

fn process_pending_serial(
    pending: Vec<PendingLibrary>,
    fail_fast: bool,
    processing: &ProcessingConfig<'_>,
    reporter: &ProgressReporter<'_>,
) -> Vec<IndexedBuildOutcome> {
    let mut processed = Vec::new();

    for pending in pending {
        let outcome = process_pending_library(pending, processing, reporter);
        let has_error = outcome
            .outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error());
        processed.push(outcome);

        if fail_fast && has_error {
            break;
        }
    }

    processed
}

fn process_pending_parallel(
    pending: Vec<PendingLibrary>,
    jobs: usize,
    processing: &ProcessingConfig<'_>,
    reporter: &ProgressReporter<'_>,
) -> Vec<IndexedBuildOutcome> {
    let threads = jobs.min(pending.len()).max(1);
    let mut groups = (0..threads).map(|_| Vec::new()).collect::<Vec<_>>();
    for (index, item) in pending.into_iter().enumerate() {
        groups[index % threads].push(item);
    }

    thread::scope(|scope| {
        let mut handles = Vec::new();
        for group in groups {
            handles.push(scope.spawn(move || {
                group
                    .into_iter()
                    .map(|pending| process_pending_library(pending, processing, reporter))
                    .collect::<Vec<_>>()
            }));
        }

        let mut processed = Vec::new();
        for handle in handles {
            processed.extend(handle.join().expect("worker thread must not panic"));
        }
        processed
    })
}

fn default_parallel_jobs() -> usize {
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
}
