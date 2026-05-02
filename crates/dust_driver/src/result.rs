use std::{path::PathBuf, sync::Arc};

use dust_diagnostics::{Diagnostic, DiagnosticFileContext};
use dust_text::{FileId, LineCol, LineIndex, TextRange};

/// One source file context attached to command diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticFile {
    /// The file identifier used by diagnostic labels in this command run.
    pub file_id: FileId,
    /// The absolute source path for the file.
    pub path: PathBuf,
    /// The original UTF-8 source text.
    pub source: Arc<str>,
    /// The precomputed line index used for line and column rendering.
    pub line_index: LineIndex,
}

impl DiagnosticFile {
    /// Creates a diagnostic file context for one source file.
    pub fn new(file_id: FileId, path: PathBuf, source: impl Into<Arc<str>>) -> Self {
        let source = source.into();
        let line_index = LineIndex::new(&source);

        Self::with_line_index(file_id, path, source, line_index)
    }

    /// Creates a diagnostic file context using a precomputed line index.
    pub fn with_line_index(
        file_id: FileId,
        path: PathBuf,
        source: impl Into<Arc<str>>,
        line_index: LineIndex,
    ) -> Self {
        let source = source.into();

        Self {
            file_id,
            path,
            source,
            line_index,
        }
    }

    /// Converts one byte range into zero-based start and end line/column pairs.
    pub fn line_cols(&self, range: TextRange) -> Option<(LineCol, LineCol)> {
        Some((
            self.line_index.line_col(range.start())?,
            self.line_index.line_col(range.end())?,
        ))
    }

    /// Returns the source text for this file.
    pub fn source_text(&self) -> &str {
        &self.source
    }

    /// Returns a diagnostics-crate render context for this file.
    pub fn render_context(&self) -> DiagnosticFileContext<'_> {
        DiagnosticFileContext::new(
            self.file_id,
            &self.path,
            self.source_text(),
            &self.line_index,
        )
    }
}

/// One generated library artifact from a build run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifact {
    /// The source Dart library path.
    pub source_path: PathBuf,
    /// The generated output path.
    pub output_path: PathBuf,
    /// Whether the output differed from the previous file contents.
    pub changed: bool,
    /// Whether the output was actually written to disk.
    pub written: bool,
    /// Whether the result came from the persistent workspace cache.
    pub cached: bool,
}

/// One checked library result from a check run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedLibrary {
    /// The source Dart library path.
    pub source_path: PathBuf,
    /// The generated output path.
    pub output_path: PathBuf,
    /// Whether the generated output is missing or stale.
    pub stale: bool,
    /// Whether the freshness result came from the persistent workspace cache.
    pub cached: bool,
}

/// One cache summary for a command run.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CacheReport {
    /// The cache file path under `.dart_tool`.
    pub path: PathBuf,
    /// The number of libraries served directly from the cache.
    pub hits: usize,
    /// The number of libraries that required pipeline work.
    pub misses: usize,
}

/// One workspace doctor report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    /// The detected package root used for library discovery.
    pub package_root: PathBuf,
    /// The resolved package configuration path.
    pub package_config_path: PathBuf,
    /// The number of candidate libraries.
    pub library_count: usize,
    /// The registered plugin names in registration order.
    pub plugin_names: Vec<String>,
    /// The discovered source library paths.
    pub libraries: Vec<PathBuf>,
}

/// One clean command summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanReport {
    /// The detected package root.
    pub package_root: PathBuf,
    /// The number of generated `.g.dart` files inspected.
    pub scanned_files: usize,
    /// The number of Dust-generated outputs removed.
    pub removed_files: usize,
    /// Whether the `.dart_tool/dust` cache directory was removed.
    pub cache_cleared: bool,
}

/// One watch-mode summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchReport {
    /// The number of poll cycles executed after the initial build.
    pub cycles: u32,
    /// The number of rebuild batches triggered by detected changes.
    pub rebuild_batches: u32,
    /// The rebuilt source libraries in rebuild order.
    pub rebuilt_libraries: Vec<PathBuf>,
}

/// The structured result returned by the Dust driver.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandResult {
    /// Diagnostics produced across all processed libraries.
    pub diagnostics: Vec<Diagnostic>,
    /// Source file contexts for diagnostics that carry labels.
    pub diagnostic_files: Vec<DiagnosticFile>,
    /// Written or skipped build artifacts.
    pub build_artifacts: Vec<BuildArtifact>,
    /// Freshness-check results.
    pub checked_libraries: Vec<CheckedLibrary>,
    /// An optional doctor report.
    pub doctor: Option<DoctorReport>,
    /// An optional clean report.
    pub clean: Option<CleanReport>,
    /// An optional watch report.
    pub watch: Option<WatchReport>,
    /// An optional cache summary.
    pub cache: Option<CacheReport>,
    /// Total elapsed wall-clock time in milliseconds.
    pub elapsed_ms: u128,
}

impl CommandResult {
    /// Returns `true` if at least one error diagnostic is present.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error())
    }
}
