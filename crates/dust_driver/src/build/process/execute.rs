use std::{sync::Arc, time::Instant};

use dust_diagnostics::Diagnostic;
use dust_emitter::WriteResult;
use dust_ir::LoweringOutcome;
use dust_parser_dart::{ParsedDartFileSurface, parse_file_with_backend};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::ResolveResult;
use dust_text::{FileId, SourceText};
use dust_workspace::SourceLibrary;

use crate::lower::lower_library_with_catalog;

use super::{
    BuildOutcome, LoweringConfig, PendingLibrary, PreprocessedLibrary, ProcessingConfig,
    emit_or_write_library,
};

/// Processes one pending library and reports progress when it finishes.
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
        pre_parse_diagnostics,
        pre_lowered,
        analysis_snapshot,
    } = pending;
    let super::LoadedLibraryInput {
        source,
        source_hash,
        tool_hash,
        checked_output_hash: _,
        previous_output_hash,
    } = input;
    let backend = TreeSitterDartBackend::new();
    let started = Instant::now();
    let mut outcome = process_library_from_source(
        file_id,
        &library,
        source,
        PreprocessedLibrary {
            parsed: pre_parsed,
            diagnostics: pre_parse_diagnostics,
            lowered: pre_lowered,
        },
        previous_output_hash,
        &backend,
        processing,
    );
    let routed = crate::build::support::route_only_analysis(&analysis_snapshot);
    outcome.artifact.routed = routed;
    outcome.analysis_snapshot = analysis_snapshot;
    let elapsed_ms = started.elapsed().as_millis();
    let had_errors = outcome
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error());
    reporter.finish(crate::build::batch::ProgressSnapshot {
        library: &library,
        cached: false,
        routed,
        written: outcome.artifact.written,
        changed: outcome.artifact.changed,
        had_errors,
        elapsed_ms,
    });

    super::IndexedBuildOutcome {
        index,
        library,
        source_hash: Some(source_hash),
        tool_hash: Some(tool_hash),
        outcome,
    }
}

/// Parses, resolves, lowers, and emits one library from already loaded source.
pub(crate) fn process_library_from_source(
    file_id: FileId,
    library: &SourceLibrary,
    source: Arc<str>,
    preprocessed: PreprocessedLibrary,
    previous_output_hash: Option<Option<u64>>,
    backend: &TreeSitterDartBackend,
    processing: &ProcessingConfig<'_>,
) -> BuildOutcome {
    let mut diagnostics = Vec::new();
    let lowered_library = if let Some(lowered) = preprocessed.lowered {
        diagnostics.extend(preprocessed.diagnostics);
        if diagnostics.iter().any(|diagnostic| diagnostic.is_error()) {
            let diagnostic_file = diagnostic_file_if_needed(
                file_id,
                library,
                Arc::clone(&source),
                diagnostics.iter().any(Diagnostic::has_labels),
            );
            return BuildOutcome::failed(library, diagnostics, diagnostic_file);
        }
        lowered
    } else {
        let parsed = preprocessed.parsed.unwrap_or_else(|| {
            let source_text = SourceText::new(file_id, Arc::clone(&source));
            let parsed = parse_file_with_backend(
                backend,
                &source_text,
                super::parse_options(processing.dart_language_version),
            );
            diagnostics.extend(parsed.diagnostics);
            parsed.library
        });
        diagnostics.extend(preprocessed.diagnostics);
        if diagnostics.iter().any(|diagnostic| diagnostic.is_error()) {
            let diagnostic_file = diagnostic_file_if_needed(
                file_id,
                library,
                Arc::clone(&source),
                diagnostics.iter().any(Diagnostic::has_labels),
            );
            return BuildOutcome::failed(library, diagnostics, diagnostic_file);
        }

        let lowering = LoweringConfig {
            package_root: processing.package_root,
            package_name: processing.package_name,
            catalog: processing.catalog,
            registry: processing.registry,
        };
        match resolve_and_lower_library(file_id, library, &parsed, &lowering, &mut diagnostics) {
            Some(library) => library,
            None => {
                let diagnostic_file = diagnostic_file_if_needed(
                    file_id,
                    library,
                    Arc::clone(&source),
                    diagnostics.iter().any(Diagnostic::has_labels),
                );
                return BuildOutcome::failed(library, diagnostics, diagnostic_file);
            }
        }
    };

    let output =
        match emit_library_output(library, &lowered_library, previous_output_hash, processing) {
            Ok(output) => output,
            Err(error) => {
                diagnostics.push(Diagnostic::error(format!(
                    "failed to write `{}`: {error}",
                    library.output_path.display()
                )));
                let diagnostic_file = diagnostic_file_if_needed(
                    file_id,
                    library,
                    Arc::clone(&source),
                    diagnostics.iter().any(Diagnostic::has_labels),
                );
                return BuildOutcome::failed(library, diagnostics, diagnostic_file);
            }
        };

    let needs_diagnostic_file = diagnostics.iter().any(Diagnostic::has_labels)
        || output.diagnostics.iter().any(Diagnostic::has_labels);
    let diagnostic_file =
        diagnostic_file_if_needed(file_id, library, source, needs_diagnostic_file);
    finish_success(library, diagnostics, diagnostic_file, output)
}

/// Builds source context only for diagnostics that need labeled rendering.
fn diagnostic_file_if_needed(
    file_id: FileId,
    library: &SourceLibrary,
    source: Arc<str>,
    needed: bool,
) -> Option<crate::result::DiagnosticFile> {
    needed.then(|| crate::result::DiagnosticFile::new(file_id, library.source_path.clone(), source))
}

/// Resolves parser output and lowers it into Dust IR.
pub(crate) fn resolve_and_lower_library(
    file_id: FileId,
    library: &SourceLibrary,
    parsed: &ParsedDartFileSurface,
    lowering: &LoweringConfig<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<dust_ir::DartFileIr> {
    let partless_configs = lowering.registry.all_partless_configs();
    let ResolveResult {
        library: mut resolved_library,
        diagnostics: resolve_diagnostics,
    } = dust_resolver::resolve_library_with_partless_configs(
        file_id,
        &workspace_relative_path(lowering.package_root, &library.source_path),
        &workspace_relative_path(lowering.package_root, &library.output_path),
        parsed,
        lowering.catalog,
        &partless_configs,
    );
    diagnostics.extend(resolve_diagnostics);

    let LoweringOutcome {
        value: mut lowered_library,
        diagnostics: lower_diagnostics,
    } = lower_library_with_catalog(&mut resolved_library, lowering.catalog);
    diagnostics.extend(lower_diagnostics);
    lowered_library.package_root = lowering.package_root.to_string_lossy().into_owned();
    lowered_library.package_name = lowering.package_name.to_owned();

    (!diagnostics.iter().any(|diagnostic| diagnostic.is_error())).then_some(lowered_library)
}

/// Emits generated output for a lowered library using the active symbol plan.
fn emit_library_output(
    library: &SourceLibrary,
    lowered_library: &dust_ir::DartFileIr,
    previous_output_hash: Option<Option<u64>>,
    processing: &ProcessingConfig<'_>,
) -> std::io::Result<WriteResult> {
    let mut plan = processing.registry.build_symbol_plan(lowered_library);
    plan.set_workspace_analysis(Arc::clone(&processing.workspace_analysis));
    emit_or_write_library(
        library,
        lowered_library,
        previous_output_hash,
        processing,
        plan,
    )
}

/// Combines emitter results with diagnostics into a successful build outcome.
fn finish_success(
    library: &SourceLibrary,
    mut diagnostics: Vec<Diagnostic>,
    diagnostic_file: Option<crate::result::DiagnosticFile>,
    output: WriteResult,
) -> BuildOutcome {
    let WriteResult {
        source: _,
        output_hash,
        symbols: _,
        diagnostics: output_diagnostics,
        changed,
        written,
        output_path: _,
        auxiliary_outputs,
    } = output;
    diagnostics.extend(output_diagnostics);

    BuildOutcome::succeeded(
        library,
        diagnostics,
        diagnostic_file,
        output_hash,
        auxiliary_outputs
            .into_iter()
            .map(|output| output.output_path)
            .collect(),
        changed,
        written,
    )
}

/// Formats a source or output path relative to the package root.
fn workspace_relative_path(package_root: &std::path::Path, path: &std::path::Path) -> String {
    path.strip_prefix(package_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
