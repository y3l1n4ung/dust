/// Per-library processing pipeline.
mod execute;
/// Emission and optional persistence of generated output.
mod output;
/// Workspace analysis scan for pending libraries.
mod scan;

use std::sync::Arc;

use dust_diagnostics::Diagnostic;
use dust_parser_dart::ParsedDartFileSurface;
use dust_plugin_api::{LibraryAnalysisSnapshot, WorkspaceAnalysis};
use dust_text::{FileId, LineIndex};
use dust_workspace::SourceLibrary;

use crate::result::{BuildArtifact, DiagnosticFile};

pub(crate) use self::{
    execute::process_pending_library, output::emit_or_write_library,
    scan::collect_workspace_analysis,
};

/// Immutable processing inputs shared by per-library workers.
#[derive(Clone)]
pub(crate) struct ProcessingConfig<'a> {
    /// Root of the Dart package being generated.
    pub(crate) package_root: &'a std::path::Path,
    /// Dart package name.
    pub(crate) package_name: &'a str,
    /// Resolver catalog built from plugin symbol ownership.
    pub(crate) catalog: &'a dust_resolver::SymbolCatalog,
    /// Active plugin registry used for validation and emission.
    pub(crate) registry: &'a dust_plugin_api::PluginRegistry,
    /// Workspace analysis snapshot visible to plugin emission.
    pub(crate) workspace_analysis: Arc<WorkspaceAnalysis>,
    /// Whether generated output should be written to disk.
    pub(crate) write_output: bool,
}

/// Result of processing one source library through Dust code generation.
pub(crate) struct BuildOutcome {
    /// Diagnostics produced while processing the library.
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// Source file and line index used to render labeled diagnostics.
    pub(crate) diagnostic_file: Option<DiagnosticFile>,
    /// User-facing artifact summary for the processed library.
    pub(crate) artifact: BuildArtifact,
    /// Hash of the expected generated output set when processing succeeded.
    pub(crate) expected_output_hash: Option<u64>,
    /// Plugin analysis facts collected for this library.
    pub(crate) analysis_snapshot: LibraryAnalysisSnapshot,
}

impl BuildOutcome {
    /// Builds a failed outcome that should invalidate the library cache entry.
    pub(crate) fn failed(
        library: &SourceLibrary,
        diagnostics: Vec<Diagnostic>,
        diagnostic_file: Option<DiagnosticFile>,
    ) -> Self {
        Self {
            diagnostics,
            diagnostic_file,
            artifact: build_artifact(library, Vec::new(), false, false, false),
            expected_output_hash: None,
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        }
    }

    /// Builds a successful outcome that can be written into the workspace cache.
    pub(crate) fn succeeded(
        library: &SourceLibrary,
        diagnostics: Vec<Diagnostic>,
        diagnostic_file: Option<DiagnosticFile>,
        expected_output_hash: u64,
        auxiliary_output_paths: Vec<std::path::PathBuf>,
        changed: bool,
        written: bool,
    ) -> Self {
        Self {
            diagnostics,
            diagnostic_file,
            artifact: build_artifact(library, auxiliary_output_paths, changed, written, false),
            expected_output_hash: Some(expected_output_hash),
            analysis_snapshot: LibraryAnalysisSnapshot::default(),
        }
    }
}

/// Build outcome paired with the source library's original discovery order.
pub(crate) struct IndexedBuildOutcome {
    /// Discovery order index used to restore deterministic output order.
    pub(crate) index: usize,
    /// Source library represented by this outcome.
    pub(crate) library: SourceLibrary,
    /// Source hash to persist on successful cache updates.
    pub(crate) source_hash: Option<u64>,
    /// Tool hash to persist on successful cache updates.
    pub(crate) tool_hash: Option<u64>,
    /// Processing result for the source library.
    pub(crate) outcome: BuildOutcome,
}

/// Loaded source and prior-output fingerprints for one source library.
#[derive(Clone)]
pub(crate) struct LoadedLibraryInput {
    /// Source text for the Dart library.
    pub(crate) source: Arc<str>,
    /// Hash of the source text.
    pub(crate) source_hash: u64,
    /// Hash of the active code generation toolchain.
    pub(crate) tool_hash: u64,
    /// Output hash trusted only when tool metadata still matches.
    pub(crate) checked_output_hash: Option<Option<u64>>,
    /// Previously generated output hash independent of current tool metadata.
    pub(crate) previous_output_hash: Option<Option<u64>>,
}

/// Pending library that needs full processing rather than a cache hit.
pub(crate) struct PendingLibrary {
    /// Discovery order index used to restore deterministic output order.
    pub(crate) index: usize,
    /// Parser file id assigned to this library.
    pub(crate) file_id: FileId,
    /// Source and output paths for this library.
    pub(crate) library: SourceLibrary,
    /// Loaded source and output fingerprint data.
    pub(crate) input: LoadedLibraryInput,
    /// Parsed library surface collected during workspace analysis, if available.
    pub(crate) pre_parsed: Option<ParsedDartFileSurface>,
    /// Plugin workspace analysis facts for this library.
    pub(crate) analysis_snapshot: LibraryAnalysisSnapshot,
}

impl PendingLibrary {
    /// Creates a pending library with empty pre-parse and analysis state.
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

/// Builds a diagnostic file handle with a precomputed line index.
pub(crate) fn build_diagnostic_file(
    file_id: FileId,
    library: &SourceLibrary,
    source: Arc<str>,
    line_index: LineIndex,
) -> DiagnosticFile {
    DiagnosticFile::with_line_index(file_id, library.source_path.clone(), source, line_index)
}

/// Builds a user-facing artifact summary for a source library.
fn build_artifact(
    library: &SourceLibrary,
    auxiliary_output_paths: Vec<std::path::PathBuf>,
    changed: bool,
    written: bool,
    cached: bool,
) -> BuildArtifact {
    BuildArtifact {
        source_path: library.source_path.clone(),
        output_path: library.output_path.clone(),
        auxiliary_output_paths,
        changed,
        written,
        cached,
        routed: false,
    }
}
