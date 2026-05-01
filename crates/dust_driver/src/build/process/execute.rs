use std::{sync::Arc, time::Instant};

use dust_diagnostics::Diagnostic;
use dust_emitter::WriteResult;
use dust_ir::LoweringOutcome;
use dust_parser_dart::{ParseOptions, ParsedLibrarySurface, parse_file_with_backend};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::ResolveResult;
use dust_text::{FileId, SourceText};
use dust_workspace::SourceLibrary;

use crate::build::support::hash_text;
use crate::lower::lower_library;

use super::{BuildOutcome, PendingLibrary, ProcessingConfig, emit_or_write_library};

pub(crate) fn process_pending_library(
    pending: PendingLibrary,
    processing: &ProcessingConfig<'_>,
    reporter: &crate::build::batch::ProgressReporter<'_>,
) -> super::IndexedBuildOutcome {
    let PendingLibrary {
        index,
        file_id,
        library,
        input,
        pre_parsed,
        analysis_snapshot,
    } = pending;
    let super::LoadedLibraryInput {
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

    super::IndexedBuildOutcome {
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

    let lowered_library =
        match resolve_and_lower_library(file_id, library, &parsed, processing, &mut diagnostics) {
            Some(library) => library,
            None => return BuildOutcome::failed(library, diagnostics),
        };

    let output = match emit_library_output(library, &lowered_library, processing) {
        Ok(output) => output,
        Err(error) => {
            diagnostics.push(Diagnostic::error(format!(
                "failed to write `{}`: {error}",
                library.output_path.display()
            )));
            return BuildOutcome::failed(library, diagnostics);
        }
    };

    finish_success(library, diagnostics, output)
}

fn resolve_and_lower_library(
    file_id: FileId,
    library: &SourceLibrary,
    parsed: &ParsedLibrarySurface,
    processing: &ProcessingConfig<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<dust_ir::LibraryIr> {
    let ResolveResult {
        library: resolved_library,
        diagnostics: resolve_diagnostics,
    } = dust_resolver::resolve_library(
        file_id,
        &library.source_path.to_string_lossy(),
        parsed,
        processing.catalog,
    );
    diagnostics.extend(resolve_diagnostics);

    let LoweringOutcome {
        value: lowered_library,
        diagnostics: lower_diagnostics,
    } = lower_library(&resolved_library);
    diagnostics.extend(lower_diagnostics);

    (!diagnostics.iter().any(|diagnostic| diagnostic.is_error())).then_some(lowered_library)
}

fn emit_library_output(
    library: &SourceLibrary,
    lowered_library: &dust_ir::LibraryIr,
    processing: &ProcessingConfig<'_>,
) -> std::io::Result<WriteResult> {
    let mut plan = processing.registry.build_symbol_plan(lowered_library);
    plan.set_workspace_analysis(Arc::clone(&processing.workspace_analysis));
    emit_or_write_library(library, lowered_library, processing, plan)
}

fn finish_success(
    library: &SourceLibrary,
    mut diagnostics: Vec<Diagnostic>,
    output: WriteResult,
) -> BuildOutcome {
    let WriteResult {
        source,
        symbols: _,
        diagnostics: output_diagnostics,
        changed,
        written,
        output_path: _,
    } = output;
    diagnostics.extend(output_diagnostics);

    BuildOutcome::succeeded(library, diagnostics, hash_text(&source), changed, written)
}
