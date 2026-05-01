use dust_diagnostics::Diagnostic;
use dust_plugin_api::LibraryAnalysisSnapshot;
use dust_workspace::SourceLibrary;

use crate::{
    build::process::{BuildOutcome, IndexedBuildOutcome},
    result::BuildArtifact,
};

pub(super) fn build_load_error(
    index: usize,
    library: &SourceLibrary,
    diagnostic: Diagnostic,
) -> IndexedBuildOutcome {
    IndexedBuildOutcome {
        index,
        library: library.clone(),
        source_hash: None,
        outcome: BuildOutcome {
            diagnostics: vec![diagnostic],
            artifact: BuildArtifact {
                source_path: library.source_path.clone(),
                output_path: library.output_path.clone(),
                changed: false,
                written: false,
                cached: false,
            },
            expected_output_hash: None,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        },
    }
}

pub(super) fn build_cached_outcome(
    index: usize,
    library: &SourceLibrary,
    expected_output_hash: u64,
    analysis_snapshot: LibraryAnalysisSnapshot,
) -> IndexedBuildOutcome {
    IndexedBuildOutcome {
        index,
        library: library.clone(),
        source_hash: None,
        outcome: BuildOutcome {
            diagnostics: Vec::new(),
            artifact: BuildArtifact {
                source_path: library.source_path.clone(),
                output_path: library.output_path.clone(),
                changed: false,
                written: false,
                cached: true,
            },
            expected_output_hash: Some(expected_output_hash),
            analysis_snapshot,
        },
    }
}
