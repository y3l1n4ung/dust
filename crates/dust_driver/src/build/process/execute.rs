use std::{sync::Arc, time::Instant};

use dust_diagnostics::Diagnostic;
use dust_emitter::WriteResult;
use dust_ir::LoweringOutcome;
use dust_parser_dart::{ParseOptions, ParsedLibrarySurface, parse_file_with_backend};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::ResolveResult;
use dust_text::{FileId, SourceText};
use dust_workspace::SourceLibrary;

use crate::build::support::hash_output_set;
use crate::lower::lower_library;

use super::{
    BuildOutcome, PendingLibrary, ProcessingConfig, build_diagnostic_file, emit_or_write_library,
};

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
    let source_text = SourceText::new(file_id, Arc::clone(&source));
    let diagnostic_file =
        build_diagnostic_file(file_id, library, source, source_text.line_index().clone());
    let parsed = pre_parsed.unwrap_or_else(|| {
        let parsed = parse_file_with_backend(backend, &source_text, ParseOptions::default());
        diagnostics.extend(parsed.diagnostics);
        parsed.library
    });

    let lowered_library =
        match resolve_and_lower_library(file_id, library, &parsed, processing, &mut diagnostics) {
            Some(library) => library,
            None => return BuildOutcome::failed(library, diagnostics, Some(diagnostic_file)),
        };

    let output = match emit_library_output(library, &lowered_library, processing) {
        Ok(output) => output,
        Err(error) => {
            diagnostics.push(Diagnostic::error(format!(
                "failed to write `{}`: {error}",
                library.output_path.display()
            )));
            return BuildOutcome::failed(library, diagnostics, Some(diagnostic_file));
        }
    };

    finish_success(library, diagnostics, Some(diagnostic_file), output)
}

fn resolve_and_lower_library(
    file_id: FileId,
    library: &SourceLibrary,
    parsed: &ParsedLibrarySurface,
    processing: &ProcessingConfig<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<dust_ir::LibraryIr> {
    let partless_configs = processing.registry.all_partless_configs();
    let ResolveResult {
        library: resolved_library,
        diagnostics: resolve_diagnostics,
    } = dust_resolver::resolve_library_with_partless_configs(
        file_id,
        &workspace_relative_path(processing.package_root, &library.source_path),
        &workspace_relative_path(processing.package_root, &library.output_path),
        parsed,
        processing.catalog,
        &partless_configs,
    );
    diagnostics.extend(resolve_diagnostics);

    let LoweringOutcome {
        value: mut lowered_library,
        diagnostics: lower_diagnostics,
    } = lower_library(&resolved_library);
    diagnostics.extend(lower_diagnostics);
    lowered_library.package_root = processing.package_root.to_string_lossy().into_owned();
    lowered_library.package_name = processing.package_name.to_owned();

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
    diagnostic_file: Option<crate::result::DiagnosticFile>,
    output: WriteResult,
) -> BuildOutcome {
    let WriteResult {
        source,
        symbols: _,
        diagnostics: output_diagnostics,
        changed,
        written,
        output_path,
        auxiliary_outputs,
    } = output;
    diagnostics.extend(output_diagnostics);

    let expected_output_hash = hash_output_set(
        std::iter::once((output_path.as_path(), source.as_str())).chain(
            auxiliary_outputs
                .iter()
                .map(|output| (output.output_path.as_path(), output.source.as_str())),
        ),
    );

    BuildOutcome::succeeded(
        library,
        diagnostics,
        diagnostic_file,
        expected_output_hash,
        auxiliary_outputs
            .into_iter()
            .map(|output| output.output_path)
            .collect(),
        changed,
        written,
    )
}

fn workspace_relative_path(package_root: &std::path::Path, path: &std::path::Path) -> String {
    path.strip_prefix(package_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
