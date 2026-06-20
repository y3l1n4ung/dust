use dust_diagnostics::Diagnostic;
use dust_plugin_api::LibraryAnalysisSnapshot;
use dust_workspace::SourceLibrary;

use crate::{
    build::process::{BuildOutcome, IndexedBuildOutcome},
    result::BuildArtifact,
};

/// Builds an indexed outcome for a library that failed during input loading.
pub(super) fn build_load_error(
    index: usize,
    library: &SourceLibrary,
    diagnostic: Diagnostic,
) -> IndexedBuildOutcome {
    IndexedBuildOutcome {
        index,
        library: library.clone(),
        source_hash: None,
        tool_hash: None,
        outcome: BuildOutcome {
            diagnostics: vec![diagnostic],
            diagnostic_file: None,
            artifact: BuildArtifact {
                source_path: library.source_path.clone(),
                output_path: library.output_path.clone(),
                auxiliary_output_paths: Vec::new(),
                changed: false,
                written: false,
                cached: false,
                routed: false,
            },
            expected_output_hash: None,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        },
    }
}

/// Builds an indexed outcome for a library reused from cache.
pub(super) fn build_cached_outcome(
    index: usize,
    library: &SourceLibrary,
    expected_output_hash: u64,
    auxiliary_output_paths: Vec<std::path::PathBuf>,
    analysis_snapshot: LibraryAnalysisSnapshot,
) -> IndexedBuildOutcome {
    let routed = crate::build::support::route_only_analysis(&analysis_snapshot);
    IndexedBuildOutcome {
        index,
        library: library.clone(),
        source_hash: None,
        tool_hash: None,
        outcome: BuildOutcome {
            diagnostics: Vec::new(),
            diagnostic_file: None,
            artifact: BuildArtifact {
                source_path: library.source_path.clone(),
                output_path: library.output_path.clone(),
                auxiliary_output_paths,
                changed: false,
                written: false,
                cached: true,
                routed,
            },
            expected_output_hash: Some(expected_output_hash),
            analysis_snapshot,
        },
    }
}
