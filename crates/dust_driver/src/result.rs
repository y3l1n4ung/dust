use std::path::PathBuf;

use dust_diagnostics::Diagnostic;

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
