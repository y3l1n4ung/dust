mod execute;
mod output;
mod scan;

use std::sync::Arc;

use dust_diagnostics::Diagnostic;
use dust_parser_dart::ParsedLibrarySurface;
use dust_plugin_api::{LibraryAnalysisSnapshot, WorkspaceAnalysis};
use dust_text::FileId;
use dust_workspace::SourceLibrary;

use crate::result::BuildArtifact;

pub(crate) use self::{
    execute::process_pending_library, output::emit_or_write_library,
    scan::collect_workspace_analysis,
};

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

impl BuildOutcome {
    pub(crate) fn failed(library: &SourceLibrary, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            diagnostics,
            artifact: build_artifact(library, false, false, false),
            expected_output_hash: None,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        }
    }

    pub(crate) fn succeeded(
        library: &SourceLibrary,
        diagnostics: Vec<Diagnostic>,
        expected_output_hash: u64,
        changed: bool,
        written: bool,
    ) -> Self {
        Self {
            diagnostics,
            artifact: build_artifact(library, changed, written, false),
            expected_output_hash: Some(expected_output_hash),
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        }
    }
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
