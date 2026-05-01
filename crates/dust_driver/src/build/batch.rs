mod execute;
mod load;
mod outcome;
mod progress;

use std::{
    path::Path,
    sync::{Arc, atomic::AtomicUsize},
};

use dust_cache::WorkspaceCache;
use dust_text::FileId;
use dust_workspace::SourceLibrary;

use crate::{
    build::{
        process::{
            IndexedBuildOutcome, PendingLibrary, ProcessingConfig, collect_workspace_analysis,
        },
        work::available_worker_count,
    },
    progress::{ProgressEvent, ProgressPhase},
    result::CacheReport,
};

pub(crate) use self::progress::{ProgressReporter, ProgressSnapshot};
use self::{
    execute::{process_pending_parallel, process_pending_serial},
    load::load_library_inputs,
    outcome::{build_cached_outcome, build_load_error},
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

pub(crate) fn prepare_and_process_batch(
    config: BatchConfig<'_>,
    libraries: &[SourceLibrary],
    cache_report: &mut CacheReport,
) -> Vec<IndexedBuildOutcome> {
    let completed = Arc::new(AtomicUsize::new(0));
    let reporter =
        ProgressReporter::new(config.progress, &completed, config.phase, libraries.len());
    reporter.started_batch();

    let loaded_results = load_library_inputs(config, libraries);
    let mut outcomes = Vec::with_capacity(libraries.len());
    let mut workspace_analysis = dust_plugin_api::WorkspaceAnalysisBuilder::default();
    let mut pending = Vec::with_capacity(libraries.len());

    for (index, input_result) in loaded_results.into_iter().enumerate() {
        let library = &libraries[index];
        let input = match input_result {
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
                continue;
            }
        };

        if let Some(entry) = config.cache.get(config.cache_root, &library.source_path) {
            if crate::build::support::matches_cache_metadata(
                entry,
                &input,
                config.package_config_hash,
                config.tool_hash,
            ) && input.checked_output_hash == Some(Some(entry.expected_output_hash))
            {
                cache_report.hits += 1;
                workspace_analysis.merge_snapshot(&entry.analysis_snapshot);
                outcomes.push(build_cached_outcome(
                    index,
                    library,
                    entry.expected_output_hash,
                    entry.analysis_snapshot.clone(),
                ));
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
        }

        cache_report.misses += 1;
        pending.push(PendingLibrary::new(
            index,
            FileId::new(config.file_id_base + index as u32),
            library.clone(),
            input,
        ));
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

    let processing = ProcessingConfig {
        catalog: config.catalog,
        registry: config.registry,
        workspace_analysis: Arc::new(workspace_analysis.build()),
        write_output: config.write_output,
    };

    let desired_jobs = available_worker_count(pending.len(), config.jobs);
    let mut processed = if config.fail_fast || desired_jobs <= 1 || pending.len() <= 1 {
        process_pending_serial(pending, config.fail_fast, &processing, &reporter)
    } else {
        process_pending_parallel(pending, desired_jobs, &processing, &reporter)
    };

    outcomes.append(&mut processed);
    outcomes.sort_by_key(|outcome| outcome.index);
    outcomes
}
