use std::{collections::HashSet, fs, time::Instant};

use dust_diagnostics::Diagnostic;
use dust_emitter::{emit_library_with_plan, write_library_with_plan};
use dust_parser_dart::{ParseOptions, parse_file_with_backend};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_text::{FileId, SourceText};
use dust_workspace::SourceLibrary;

use crate::{build::support::hash_text, lower::lower_library, result::BuildArtifact};

#[derive(Clone, Copy)]
pub(crate) struct ProcessingConfig<'a> {
    pub(crate) catalog: &'a dust_resolver::SymbolCatalog,
    pub(crate) registry: &'a dust_plugin_api::PluginRegistry,
    pub(crate) copyable_types: &'a HashSet<String>,
    pub(crate) write_output: bool,
}

pub(crate) struct BuildOutcome {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) artifact: BuildArtifact,
    pub(crate) expected_output_hash: Option<u64>,
}

pub(crate) struct IndexedBuildOutcome {
    pub(crate) index: usize,
    pub(crate) library: SourceLibrary,
    pub(crate) source_hash: Option<u64>,
    pub(crate) outcome: BuildOutcome,
}

pub(crate) struct LoadedLibraryInput {
    pub(crate) source: String,
    pub(crate) source_hash: u64,
    pub(crate) output_hash: Option<u64>,
}

pub(crate) struct PendingLibrary {
    pub(crate) index: usize,
    pub(crate) file_id: FileId,
    pub(crate) library: SourceLibrary,
    pub(crate) input: LoadedLibraryInput,
}

pub(crate) fn process_pending_library(
    pending: PendingLibrary,
    processing: &ProcessingConfig<'_>,
    reporter: &crate::build::batch::ProgressReporter<'_>,
) -> IndexedBuildOutcome {
    let backend = TreeSitterDartBackend::new();
    let started = Instant::now();
    let outcome = process_library_from_source(
        pending.file_id,
        &pending.library,
        pending.input.source,
        &backend,
        processing,
    );
    let elapsed_ms = started.elapsed().as_millis();
    let had_errors = outcome
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error());
    reporter.finish(crate::build::batch::ProgressSnapshot {
        library: &pending.library,
        cached: false,
        written: outcome.artifact.written,
        changed: outcome.artifact.changed,
        had_errors,
        elapsed_ms,
    });

    IndexedBuildOutcome {
        index: pending.index,
        library: pending.library,
        source_hash: Some(pending.input.source_hash),
        outcome,
    }
}

pub(crate) fn process_library_from_source(
    file_id: FileId,
    library: &SourceLibrary,
    source: String,
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
    let parsed = parse_file_with_backend(backend, &source_text, ParseOptions::default());
    diagnostics.extend(parsed.diagnostics.clone());

    let resolved = dust_resolver::resolve_library(
        file_id,
        &library.source_path.to_string_lossy(),
        &parsed.library,
        processing.catalog,
    );
    diagnostics.extend(resolved.diagnostics.clone());

    let lowered = lower_library(&resolved.library);
    diagnostics.extend(lowered.diagnostics.clone());

    if diagnostics.iter().any(|diagnostic| diagnostic.is_error()) {
        return BuildOutcome {
            diagnostics,
            artifact,
            expected_output_hash: None,
        };
    }

    let mut plan = processing.registry.build_symbol_plan(&lowered.value);
    plan.extend_copyable_types(processing.copyable_types.iter().cloned());

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
    }
}

pub(crate) fn collect_workspace_copyable_types(
    loaded_sources: &[(FileId, SourceLibrary, String)],
    catalog: &dust_resolver::SymbolCatalog,
) -> HashSet<String> {
    let backend = TreeSitterDartBackend::new();

    loaded_sources
        .iter()
        .flat_map(|(file_id, library, source)| {
            let source_text = SourceText::new(*file_id, source.clone());
            let parsed = parse_file_with_backend(&backend, &source_text, ParseOptions::default());
            let resolved = dust_resolver::resolve_library(
                *file_id,
                &library.source_path.to_string_lossy(),
                &parsed.library,
                catalog,
            );

            resolved
                .library
                .classes
                .into_iter()
                .filter(|class| {
                    class
                        .traits
                        .iter()
                        .any(|trait_app| trait_app.symbol.0 == "derive_annotation::CopyWith")
                })
                .map(|class| class.name)
                .collect::<Vec<_>>()
        })
        .collect()
}
