/// Serial and parallel processing of uncached libraries.
mod execute;
/// Parallel source and output fingerprint loading.
mod load;
/// Constructors for cached and load-error build outcomes.
mod outcome;
/// Progress callback adapter for batch execution.
mod progress;

use std::{
    path::Path,
    sync::{Arc, atomic::AtomicUsize},
};

use dust_cache::WorkspaceCache;
use dust_dart_syntax::DartLanguageVersion;
use dust_plugin_api::{PACKAGE_FEATURE_FLUTTER, PACKAGE_FEATURES_ANALYSIS_KEY};
use dust_text::FileId;
use dust_workspace::SourceLibrary;

use crate::{
    build::{
        process::{
            IndexedBuildOutcome, PendingLibrary, ProcessingConfig, collect_workspace_analysis,
        },
        support::CodegenToolHash,
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

/// Callback signature used by build and watch progress reporting.
pub(crate) type ProgressCallback<'a> = dyn Fn(ProgressEvent) + Send + Sync + 'a;

/// Shared inputs needed to load, cache-check, analyze, and process a library batch.
#[derive(Clone, Copy)]
pub(crate) struct BatchConfig<'a> {
    /// Root directory that stores Dust cache metadata.
    pub(crate) cache_root: &'a Path,
    /// Package root used to resolve source-relative output paths.
    pub(crate) package_root: &'a Path,
    /// Dart package name used by plugin workspace analysis.
    pub(crate) package_name: &'a str,
    /// Whether the package is a Flutter package.
    pub(crate) is_flutter_package: bool,
    /// Workspace-level Dart language version for source files without an
    /// explicit `// @dart = x.y` comment.
    pub(crate) dart_language_version: DartLanguageVersion,
    /// Hash of package and Dust configuration files.
    pub(crate) package_config_hash: u64,
    /// Hash of Dust codegen logic and active plugin set.
    pub(crate) tool_hash: CodegenToolHash,
    /// Current workspace cache used for hit detection.
    pub(crate) cache: &'a WorkspaceCache,
    /// Resolver catalog built from active plugin symbols.
    pub(crate) catalog: &'a dust_resolver::SymbolCatalog,
    /// Active plugin registry for validation and emission.
    pub(crate) registry: &'a dust_plugin_api::PluginRegistry,
    /// Whether generated files should be persisted to disk.
    pub(crate) write_output: bool,
    /// Whether processing should stop after the first error.
    pub(crate) fail_fast: bool,
    /// Optional caller-specified worker limit.
    pub(crate) jobs: Option<usize>,
    /// Starting file id assigned to parsed source libraries.
    pub(crate) file_id_base: u32,
    /// Progress phase associated with this batch.
    pub(crate) phase: ProgressPhase,
    /// Optional progress event callback.
    pub(crate) progress: Option<&'a ProgressCallback<'a>>,
}

/// Result of batch processing including workspace analysis metadata.
pub(crate) struct BatchResult {
    /// Per-library build outcomes in discovery order.
    pub(crate) outcomes: Vec<IndexedBuildOutcome>,
    /// Hash of the workspace analysis active during this batch.
    pub(crate) workspace_analysis_hash: u64,
}

/// A tentative cache hit awaiting workspace analysis hash verification.
struct TentativeHit<'a> {
    /// Discovery order index for deterministic output ordering.
    index: usize,
    /// Source library represented by this tentative hit.
    library: &'a SourceLibrary,
    /// Cached entry to verify against the current workspace analysis hash.
    entry: dust_cache::CacheEntry,
    /// Loaded source and output fingerprints for demotion to pending.
    input: crate::build::process::LoadedLibraryInput,
}

/// Loads inputs, reuses cache hits, runs workspace analysis, and processes misses.
pub(crate) fn prepare_and_process_batch(
    config: BatchConfig<'_>,
    libraries: &[SourceLibrary],
    cache_report: &mut CacheReport,
) -> BatchResult {
    let completed = Arc::new(AtomicUsize::new(0));
    let reporter =
        ProgressReporter::new(config.progress, &completed, config.phase, libraries.len());
    reporter.started_batch();

    let loaded_results = load_library_inputs(config, libraries);
    let mut outcomes = Vec::with_capacity(libraries.len());
    let mut workspace_analysis = dust_plugin_api::WorkspaceAnalysisBuilder::default();
    if config.is_flutter_package {
        workspace_analysis
            .add_string_set_value(PACKAGE_FEATURES_ANALYSIS_KEY, PACKAGE_FEATURE_FLUTTER);
    }
    let mut pending = Vec::with_capacity(libraries.len());
    let mut tentative_hits: Vec<TentativeHit<'_>> = Vec::new();

    // Pass 1: separate tentative cache hits from definite misses.
    for (index, input_result) in loaded_results.into_iter().enumerate() {
        let library = &libraries[index];
        let input = match input_result {
            Ok(input) => input,
            Err(diagnostic) => {
                outcomes.push(build_load_error(index, library, diagnostic));
                reporter.finish(ProgressSnapshot {
                    library,
                    cached: false,
                    routed: false,
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
            ) && input.checked_output_hash == Some(Some(entry.expected_output_hash))
            {
                workspace_analysis.merge_snapshot(&entry.analysis_snapshot);
                tentative_hits.push(TentativeHit {
                    index,
                    library,
                    entry: entry.clone(),
                    input,
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

    // Scan cache-miss libraries and merge their analysis facts.
    let workspace_result = collect_workspace_analysis(
        &pending,
        config.package_root,
        config.package_name,
        config.dart_language_version,
        config.catalog,
        config.registry,
    );
    workspace_analysis.merge(workspace_result.analysis);
    for ((((pending, pre_parsed), pre_parse_diagnostics), pre_lowered), analysis_snapshot) in
        pending
            .iter_mut()
            .zip(workspace_result.pre_parsed)
            .zip(workspace_result.pre_parse_diagnostics)
            .zip(workspace_result.pre_lowered)
            .zip(workspace_result.snapshots)
    {
        pending.pre_parsed = pre_parsed;
        pending.pre_parse_diagnostics = pre_parse_diagnostics;
        pending.pre_lowered = pre_lowered;
        pending.analysis_snapshot = analysis_snapshot;
    }

    // Build the final workspace analysis and compute its content hash.
    let built_analysis = workspace_analysis.build();
    let workspace_analysis_hash = built_analysis.content_hash();

    // Pass 2: verify tentative hits against the workspace analysis hash.
    for hit in tentative_hits {
        if hit.entry.workspace_analysis_hash == workspace_analysis_hash {
            cache_report.hits += 1;
            outcomes.push(build_cached_outcome(
                hit.index,
                hit.library,
                hit.entry.expected_output_hash,
                hit.entry.auxiliary_output_paths.clone(),
                hit.entry.suppress_primary_output,
                hit.entry.analysis_snapshot.clone(),
            ));
            reporter.finish(ProgressSnapshot {
                library: hit.library,
                cached: true,
                routed: crate::build::support::route_only_analysis(&hit.entry.analysis_snapshot),
                written: false,
                changed: false,
                had_errors: false,
                elapsed_ms: 0,
            });
        } else {
            cache_report.misses += 1;
            let mut demoted = PendingLibrary::new(
                hit.index,
                FileId::new(config.file_id_base + hit.index as u32),
                hit.library.clone(),
                hit.input,
            );
            demoted.analysis_snapshot = hit.entry.analysis_snapshot;
            pending.push(demoted);
        }
    }

    let processing = ProcessingConfig {
        package_root: config.package_root,
        package_name: config.package_name,
        dart_language_version: config.dart_language_version,
        catalog: config.catalog,
        registry: config.registry,
        workspace_analysis: Arc::new(built_analysis),
        write_output: config.write_output,
    };

    let desired_jobs = available_worker_count(pending.len(), config.jobs);
    let mut processed = if desired_jobs <= 1 || pending.len() <= 1 {
        process_pending_serial(pending, config.fail_fast, &processing, &reporter)
    } else {
        process_pending_parallel(
            pending,
            desired_jobs,
            config.fail_fast,
            &processing,
            &reporter,
        )
    };

    outcomes.append(&mut processed);
    outcomes.sort_by_key(|outcome| outcome.index);
    BatchResult {
        outcomes,
        workspace_analysis_hash,
    }
}
