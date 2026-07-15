use std::sync::Arc;

use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_parser_dart::{ParseOptions, ParsedDartFileSurface, parse_file_with_backend};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_plugin_api::{LibraryAnalysisSnapshot, PluginRegistry, WorkspaceAnalysisBuilder};
use dust_resolver::SymbolCatalog;
use dust_text::SourceText;

use crate::build::work::{available_worker_count, round_robin_groups};

use super::{LoweringConfig, PendingLibrary, execute::resolve_and_lower_library};

/// Workspace facts and preprocessed libraries collected during the scan phase.
pub(crate) struct WorkspaceAnalysisResult {
    /// Plugin facts merged across all pending libraries.
    pub(crate) analysis: WorkspaceAnalysisBuilder,
    /// Parsed surfaces retained for fallback processing.
    pub(crate) pre_parsed: Vec<Option<ParsedDartFileSurface>>,
    /// Lowered IR retained for normal processing and emission.
    pub(crate) pre_lowered: Vec<Option<DartFileIr>>,
    /// Per-library plugin facts persisted into cache entries.
    pub(crate) snapshots: Vec<LibraryAnalysisSnapshot>,
}

/// Collects plugin workspace analysis for pending libraries in parallel.
pub(crate) fn collect_workspace_analysis(
    pending: &[PendingLibrary],
    package_root: &std::path::Path,
    package_name: &str,
    catalog: &SymbolCatalog,
    registry: &PluginRegistry,
) -> WorkspaceAnalysisResult {
    if pending.is_empty() {
        return WorkspaceAnalysisResult {
            analysis: WorkspaceAnalysisBuilder::default(),
            pre_parsed: Vec::new(),
            pre_lowered: Vec::new(),
            snapshots: Vec::new(),
        };
    }

    let threads = available_worker_count(pending.len(), None);
    let groups = round_robin_groups(pending.iter().enumerate(), threads);
    let lowering = LoweringConfig {
        package_root,
        package_name,
        catalog,
        registry,
    };
    let lowering = &lowering;

    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(groups.len());
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
                    let lowered = lower_for_workspace_analysis(
                        pending,
                        &parsed.library,
                        &parsed.diagnostics,
                        lowering,
                    );
                    if let Some(lowered) = lowered.as_ref() {
                        registry.collect_workspace_analysis_ir(lowered, &mut library_analysis);
                    }
                    let analysis_snapshot = library_analysis.snapshot();
                    local_analysis.merge(library_analysis);
                    local_results.push((index, analysis_snapshot, parsed.library, lowered));
                }

                (local_analysis, local_results)
            }));
        }

        let mut workspace_analysis = WorkspaceAnalysisBuilder::default();
        let mut ordered_surfaces = vec![None; pending.len()];
        let mut ordered_lowered = vec![None; pending.len()];
        let mut analysis_snapshots = vec![LibraryAnalysisSnapshot::default(); pending.len()];

        for handle in handles {
            let (local_analysis, local) = handle.join().expect("scan thread must not panic");
            workspace_analysis.merge(local_analysis);
            for (index, analysis_snapshot, parsed, lowered) in local {
                analysis_snapshots[index] = analysis_snapshot;
                ordered_surfaces[index] = Some(parsed);
                ordered_lowered[index] = lowered;
            }
        }

        WorkspaceAnalysisResult {
            analysis: workspace_analysis,
            pre_parsed: ordered_surfaces,
            pre_lowered: ordered_lowered,
            snapshots: analysis_snapshots,
        }
    })
}

/// Lowers a parsed library once so IR-native workspace analysis can feed emission.
fn lower_for_workspace_analysis(
    pending: &PendingLibrary,
    parsed: &ParsedDartFileSurface,
    parse_diagnostics: &[Diagnostic],
    lowering: &LoweringConfig<'_>,
) -> Option<DartFileIr> {
    if !parse_diagnostics.is_empty() {
        return None;
    }

    let mut diagnostics = Vec::new();
    let lowered = resolve_and_lower_library(
        pending.file_id,
        &pending.library,
        parsed,
        lowering,
        &mut diagnostics,
    )?;
    diagnostics.is_empty().then_some(lowered)
}
