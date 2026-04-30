use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use dust_cache::WorkspaceCache;
use dust_diagnostics::Diagnostic;
use dust_text::FileId;
use dust_workspace::SourceLibrary;

use crate::{
    build::{
        process::{
            BuildOutcome, IndexedBuildOutcome, PendingLibrary, ProcessingConfig,
            collect_workspace_copyable_types, process_pending_library,
        },
        support::{load_library_input, matches_cache},
    },
    progress::{ProgressEvent, ProgressPhase},
    result::{BuildArtifact, CacheReport},
};

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

#[derive(Clone, Copy)]
pub(crate) struct ProgressReporter<'a> {
    progress: Option<&'a ProgressCallback<'a>>,
    completed: &'a AtomicUsize,
    phase: ProgressPhase,
    total: usize,
}

pub(crate) struct ProgressSnapshot<'a> {
    pub(crate) library: &'a SourceLibrary,
    pub(crate) cached: bool,
    pub(crate) written: bool,
    pub(crate) changed: bool,
    pub(crate) had_errors: bool,
    pub(crate) elapsed_ms: u128,
}

impl ProgressReporter<'_> {
    pub(crate) fn started_batch(&self) {
        if let Some(progress) = self.progress {
            progress(ProgressEvent::StartedBatch {
                phase: self.phase,
                total: self.total,
            });
        }
    }

    pub(crate) fn finish(&self, snapshot: ProgressSnapshot<'_>) {
        if let Some(progress) = self.progress {
            let completed = self.completed.fetch_add(1, Ordering::SeqCst) + 1;
            progress(ProgressEvent::FinishedLibrary {
                phase: self.phase,
                completed,
                total: self.total,
                source_path: snapshot.library.source_path.clone(),
                cached: snapshot.cached,
                written: snapshot.written,
                changed: snapshot.changed,
                had_errors: snapshot.had_errors,
                elapsed_ms: snapshot.elapsed_ms,
            });
        }
    }
}

pub(crate) fn prepare_and_process_batch(
    config: BatchConfig<'_>,
    libraries: &[SourceLibrary],
    cache_report: &mut CacheReport,
) -> Vec<IndexedBuildOutcome> {
    let completed = Arc::new(AtomicUsize::new(0));
    let reporter = ProgressReporter {
        progress: config.progress,
        completed: &completed,
        phase: config.phase,
        total: libraries.len(),
    };
    reporter.started_batch();

    let mut outcomes = Vec::new();
    let mut pending = Vec::new();
    let mut loaded_sources = Vec::new();

    for (index, library) in libraries.iter().enumerate() {
        let input = match load_library_input(library) {
            Ok(input) => input,
            Err(diagnostic) => {
                outcomes.push(build_load_error(index, library, diagnostic));
                reporter.finish(ProgressSnapshot {
                    library,
                    cached: false,
                    written: false,
                    changed: false,
                    had_errors: true,
                    elapsed_ms: 0,
                });
                if config.fail_fast {
                    break;
                }
                continue;
            }
        };
        loaded_sources.push((
            FileId::new(config.file_id_base + index as u32),
            library.clone(),
            input.source.clone(),
        ));

        if matches_cache(
            config.cache,
            config.cache_root,
            library,
            &input,
            config.package_config_hash,
            config.tool_hash,
        ) {
            cache_report.hits += 1;
            outcomes.push(build_cached_outcome(index, library, input.output_hash));
            reporter.finish(ProgressSnapshot {
                library,
                cached: true,
                written: false,
                changed: false,
                had_errors: false,
                elapsed_ms: 0,
            });
            continue;
        }

        cache_report.misses += 1;
        pending.push(PendingLibrary {
            index,
            file_id: FileId::new(config.file_id_base + index as u32),
            library: library.clone(),
            input,
        });
    }

    let copyable_types = collect_workspace_copyable_types(&loaded_sources, config.catalog);
    let processing = ProcessingConfig {
        catalog: config.catalog,
        registry: config.registry,
        copyable_types: &copyable_types,
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

fn build_load_error(
    index: usize,
    library: &SourceLibrary,
    diagnostic: Diagnostic,
) -> IndexedBuildOutcome {
    IndexedBuildOutcome {
        index,
        library: library.clone(),
        source_hash: None,
        outcome: BuildOutcome {
            diagnostics: vec![diagnostic],
            artifact: BuildArtifact {
                source_path: library.source_path.clone(),
                output_path: library.output_path.clone(),
                changed: false,
                written: false,
                cached: false,
            },
            expected_output_hash: None,
        },
    }
}

fn build_cached_outcome(
    index: usize,
    library: &SourceLibrary,
    output_hash: Option<u64>,
) -> IndexedBuildOutcome {
    IndexedBuildOutcome {
        index,
        library: library.clone(),
        source_hash: None,
        outcome: BuildOutcome {
            diagnostics: Vec::new(),
            artifact: BuildArtifact {
                source_path: library.source_path.clone(),
                output_path: library.output_path.clone(),
                changed: false,
                written: false,
                cached: true,
            },
            expected_output_hash: output_hash,
        },
    }
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
