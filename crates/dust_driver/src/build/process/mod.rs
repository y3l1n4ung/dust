mod scan;

use std::{fs, io, path::Path, sync::Arc, time::Instant};

use dust_diagnostics::Diagnostic;
use dust_emitter::{WriteResult, emit_library_with_plan, persist_emit_result};
use dust_ir::LoweringOutcome;
use dust_parser_dart::{ParseOptions, ParsedLibrarySurface, parse_file_with_backend};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_plugin_api::{LibraryAnalysisSnapshot, WorkspaceAnalysis};
use dust_resolver::ResolveResult;
use dust_text::{FileId, SourceText};
use dust_workspace::SourceLibrary;

use crate::{build::support::hash_text, lower::lower_library, result::BuildArtifact};

pub(crate) use self::scan::collect_workspace_analysis;

#[derive(Clone)]
pub(crate) struct ProcessingConfig<'a> {
    pub(crate) catalog: &'a dust_resolver::SymbolCatalog,
    pub(crate) registry: &'a dust_plugin_api::PluginRegistry,
    pub(crate) workspace_analysis: Arc<WorkspaceAnalysis>,
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

impl PendingLibrary {
    pub(crate) fn new(
        index: usize,
        file_id: FileId,
        library: SourceLibrary,
        input: LoadedLibraryInput,
    ) -> Self {
        Self {
            index,
            file_id,
            library,
            input,
            pre_parsed: None,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        }
    }
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
    let source_text = SourceText::new(file_id, source);
    let parsed = pre_parsed.unwrap_or_else(|| {
        let parsed = parse_file_with_backend(backend, &source_text, ParseOptions::default());
        diagnostics.extend(parsed.diagnostics);
        parsed.library
    });

    let ResolveResult {
        library: resolved_library,
        diagnostics: resolve_diagnostics,
    } = dust_resolver::resolve_library(
        file_id,
        &library.source_path.to_string_lossy(),
        &parsed,
        processing.catalog,
    );
    diagnostics.extend(resolve_diagnostics);

    let LoweringOutcome {
        value: lowered_library,
        diagnostics: lower_diagnostics,
    } = lower_library(&resolved_library);
    diagnostics.extend(lower_diagnostics);

    if diagnostics.iter().any(|diagnostic| diagnostic.is_error()) {
        return BuildOutcome {
            diagnostics,
            artifact: build_artifact(library, false, false, false),
            expected_output_hash: None,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        };
    }

    let mut plan = processing.registry.build_symbol_plan(&lowered_library);
    plan.set_workspace_analysis(Arc::clone(&processing.workspace_analysis));

    let output = match emit_or_write_library(library, &lowered_library, processing, plan) {
        Ok(output) => output,
        Err(error) => {
            diagnostics.push(Diagnostic::error(format!(
                "failed to write `{}`: {error}",
                library.output_path.display()
            )));
            return BuildOutcome {
                diagnostics,
                artifact: build_artifact(library, false, false, false),
                expected_output_hash: None,
                analysis_snapshot: LibraryAnalysisSnapshot::default(),
            };
        }
    };

    let WriteResult {
        source,
        symbols: _,
        diagnostics: output_diagnostics,
        changed,
        written,
        output_path: _,
    } = output;
    diagnostics.extend(output_diagnostics);

    BuildOutcome {
        diagnostics,
        artifact: build_artifact(library, changed, written, false),
        expected_output_hash: Some(hash_text(&source)),
        analysis_snapshot: LibraryAnalysisSnapshot::default(),
    }
}

fn emit_or_write_library(
    library: &SourceLibrary,
    lowered_library: &dust_ir::LibraryIr,
    processing: &ProcessingConfig<'_>,
    plan: dust_plugin_api::SymbolPlan,
) -> std::io::Result<WriteResult> {
    let previous = read_previous_output(&library.output_path, processing.write_output)?;
    let emitted = emit_library_with_plan(
        lowered_library,
        processing.registry,
        plan,
        previous.as_deref(),
    );

    if processing.write_output {
        persist_emit_result(library.output_path.clone(), emitted)
    } else {
        Ok(WriteResult {
            source: emitted.source,
            symbols: emitted.symbols,
            diagnostics: emitted.diagnostics,
            changed: emitted.changed,
            written: false,
            output_path: library.output_path.clone(),
        })
    }
}

fn read_previous_output(path: &Path, strict: bool) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) if strict => Err(error),
        Err(_) => Ok(None),
    }
}

fn build_artifact(
    library: &SourceLibrary,
    changed: bool,
    written: bool,
    cached: bool,
) -> BuildArtifact {
    BuildArtifact {
        source_path: library.source_path.clone(),
        output_path: library.output_path.clone(),
        changed,
        written,
        cached,
    }
}
