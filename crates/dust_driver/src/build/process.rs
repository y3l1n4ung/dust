use std::{fs, sync::Arc, time::Instant};

use dust_diagnostics::Diagnostic;
use dust_emitter::{emit_library_with_plan, write_library_with_plan};
use dust_parser_dart::{ParseOptions, ParsedLibrarySurface, parse_file_with_backend};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_plugin_api::{
    LibraryAnalysisSnapshot, PluginRegistry, WorkspaceAnalysis, WorkspaceAnalysisBuilder,
};
use dust_text::{FileId, SourceText};
use dust_workspace::SourceLibrary;

use crate::{build::support::hash_text, lower::lower_library, result::BuildArtifact};

#[derive(Clone, Copy)]
pub(crate) struct ProcessingConfig<'a> {
    pub(crate) catalog: &'a dust_resolver::SymbolCatalog,
    pub(crate) registry: &'a dust_plugin_api::PluginRegistry,
    pub(crate) workspace_analysis: &'a Arc<WorkspaceAnalysis>,
    pub(crate) write_output: bool,
}

pub(crate) struct BuildOutcome {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) artifact: BuildArtifact,
    pub(crate) expected_output_hash: Option<u64>,
    pub(crate) analysis_snapshot: LibraryAnalysisSnapshot,
}

pub(crate) struct IndexedBuildOutcome {
    pub(crate) index: usize,
    pub(crate) library: SourceLibrary,
    pub(crate) source_hash: Option<u64>,
    pub(crate) outcome: BuildOutcome,
}

#[derive(Clone)]
pub(crate) struct LoadedLibraryInput {
    pub(crate) source: Arc<str>,
    pub(crate) source_hash: u64,
    pub(crate) checked_output_hash: Option<Option<u64>>,
}

pub(crate) struct PendingLibrary {
    pub(crate) index: usize,
    pub(crate) file_id: FileId,
    pub(crate) library: SourceLibrary,
    pub(crate) input: LoadedLibraryInput,
    pub(crate) pre_parsed: Option<ParsedLibrarySurface>,
    pub(crate) analysis_snapshot: LibraryAnalysisSnapshot,
}

pub(crate) fn process_pending_library(
    pending: PendingLibrary,
    processing: &ProcessingConfig<'_>,
    reporter: &crate::build::batch::ProgressReporter<'_>,
) -> IndexedBuildOutcome {
    let PendingLibrary {
        index,
        file_id,
        library,
        input,
        pre_parsed,
        analysis_snapshot,
    } = pending;
    let LoadedLibraryInput {
        source,
        source_hash,
        checked_output_hash: _,
    } = input;
    let backend = TreeSitterDartBackend::new();
    let started = Instant::now();
    let mut outcome =
        process_library_from_source(file_id, &library, source, pre_parsed, &backend, processing);
    outcome.analysis_snapshot = analysis_snapshot;
    let elapsed_ms = started.elapsed().as_millis();
    let had_errors = outcome
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error());
    reporter.finish(crate::build::batch::ProgressSnapshot {
        library: &library,
        cached: false,
        written: outcome.artifact.written,
        changed: outcome.artifact.changed,
        had_errors,
        elapsed_ms,
    });

    IndexedBuildOutcome {
        index,
        library,
        source_hash: Some(source_hash),
        outcome,
    }
}

pub(crate) fn process_library_from_source(
    file_id: FileId,
    library: &SourceLibrary,
    source: Arc<str>,
    pre_parsed: Option<ParsedLibrarySurface>,
    backend: &TreeSitterDartBackend,
    processing: &ProcessingConfig<'_>,
) -> BuildOutcome {
    let mut diagnostics = Vec::new();
    let artifact = BuildArtifact {
        source_path: library.source_path.clone(),
        output_path: library.output_path.clone(),
        changed: false,
        written: false,
        cached: false,
    };
    let source_text = SourceText::new(file_id, source);
    let parsed = pre_parsed.unwrap_or_else(|| {
        let parsed = parse_file_with_backend(backend, &source_text, ParseOptions::default());
        diagnostics.extend(parsed.diagnostics);
        parsed.library
    });
    let resolved = dust_resolver::resolve_library(
        file_id,
        &library.source_path.to_string_lossy(),
        &parsed,
        processing.catalog,
    );
    diagnostics.extend(resolved.diagnostics.iter().cloned());
    let resolved = resolved.library;

    let lowered = lower_library(&resolved);
    diagnostics.extend(lowered.diagnostics.clone());

    if diagnostics.iter().any(|diagnostic| diagnostic.is_error()) {
        return BuildOutcome {
            diagnostics,
            artifact,
            expected_output_hash: None,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        };
    }

    let mut plan = processing.registry.build_symbol_plan(&lowered.value);
    plan.set_workspace_analysis(Arc::clone(processing.workspace_analysis));

    let output = if processing.write_output {
        match write_library_with_plan(&lowered.value, processing.registry, plan.clone()) {
            Ok(output) => output,
            Err(error) => {
                diagnostics.push(Diagnostic::error(format!(
                    "failed to write `{}`: {error}",
                    library.output_path.display()
                )));
                return BuildOutcome {
                    diagnostics,
                    artifact,
                    expected_output_hash: None,
                    analysis_snapshot: LibraryAnalysisSnapshot::default(),
                };
            }
        }
    } else {
        let previous = fs::read_to_string(&library.output_path).ok();
        let emitted = emit_library_with_plan(
            &lowered.value,
            processing.registry,
            plan,
            previous.as_deref(),
        );
        dust_emitter::WriteResult {
            source: emitted.source,
            symbols: emitted.symbols,
            diagnostics: emitted.diagnostics,
            changed: emitted.changed,
            written: false,
            output_path: library.output_path.clone(),
        }
    };

    diagnostics.extend(output.diagnostics.clone());

    BuildOutcome {
        diagnostics,
        artifact: BuildArtifact {
            source_path: library.source_path.clone(),
            output_path: library.output_path.clone(),
            changed: output.changed,
            written: output.written,
            cached: false,
        },
        expected_output_hash: Some(hash_text(&output.source)),
        analysis_snapshot: LibraryAnalysisSnapshot::default(),
    }
}

pub(crate) fn collect_workspace_analysis(
    pending: &[PendingLibrary],
    registry: &PluginRegistry,
) -> (
    WorkspaceAnalysisBuilder,
    Vec<Option<ParsedLibrarySurface>>,
    Vec<LibraryAnalysisSnapshot>,
) {
    use std::thread;

    if pending.is_empty() {
        return (WorkspaceAnalysisBuilder::default(), Vec::new(), Vec::new());
    }

    let threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(pending.len())
        .max(1);

    let mut groups = (0..threads).map(|_| Vec::new()).collect::<Vec<_>>();
    for (index, item) in pending.iter().enumerate() {
        groups[index % threads].push((index, item));
    }

    thread::scope(|scope| {
        let mut handles = Vec::new();
        for group in groups {
            handles.push(scope.spawn(move || {
                let backend = TreeSitterDartBackend::new();
                let mut local_analysis = WorkspaceAnalysisBuilder::default();
                let mut local_results = Vec::new();

                for (index, pending) in group {
                    let source_text =
                        SourceText::new(pending.file_id, Arc::clone(&pending.input.source));
                    let parsed =
                        parse_file_with_backend(&backend, &source_text, ParseOptions::default());
                    let mut library_analysis = WorkspaceAnalysisBuilder::default();
                    registry.collect_workspace_analysis(&parsed.library, &mut library_analysis);
                    let analysis_snapshot = library_analysis.snapshot();
                    local_analysis.merge(library_analysis);
                    local_results.push((index, analysis_snapshot, Some(parsed.library)));
                }
                (local_analysis, local_results)
            }));
        }

        let mut workspace_analysis = WorkspaceAnalysisBuilder::default();
        let mut ordered_surfaces = vec![None; pending.len()];
        let mut analysis_snapshots = vec![LibraryAnalysisSnapshot::default(); pending.len()];

        for handle in handles {
            let (local_analysis, local) = handle.join().expect("scan thread must not panic");
            workspace_analysis.merge(local_analysis);
            for (index, analysis_snapshot, parsed) in local {
                analysis_snapshots[index] = analysis_snapshot;
                ordered_surfaces[index] = parsed;
            }
        }
        (workspace_analysis, ordered_surfaces, analysis_snapshots)
    })
}
