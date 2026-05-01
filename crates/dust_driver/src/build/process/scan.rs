use std::{sync::Arc, thread};

use dust_parser_dart::{ParseOptions, ParsedLibrarySurface, parse_file_with_backend};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_plugin_api::{LibraryAnalysisSnapshot, PluginRegistry, WorkspaceAnalysisBuilder};
use dust_text::SourceText;

use super::PendingLibrary;

pub(crate) fn collect_workspace_analysis(
    pending: &[PendingLibrary],
    registry: &PluginRegistry,
) -> (
    WorkspaceAnalysisBuilder,
    Vec<Option<ParsedLibrarySurface>>,
    Vec<LibraryAnalysisSnapshot>,
) {
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
                    local_results.push((index, analysis_snapshot, parsed.library));
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
                ordered_surfaces[index] = Some(parsed);
            }
        }

        (workspace_analysis, ordered_surfaces, analysis_snapshots)
    })
}
