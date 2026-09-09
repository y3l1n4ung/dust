//! Per-command report payloads carried back to the CLI.

use super::*;

/// Workspace i18n ARB build report.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct I18nBuildReport {
    /// Number of Dart source files scanned.
    pub scanned_files: usize,
    /// Number of unique static translation keys found.
    pub keys: usize,
    /// Number of ARB files inspected for configured locales and namespaces.
    pub arb_files: usize,
    /// Number of ARB files changed, or planned in preview mode.
    pub changed_files: usize,
    /// Number of message entries added across all ARB files.
    pub added_messages: usize,
    /// Number of existing fallback-locale messages synced from `defaultText`.
    pub synced_messages: usize,
    /// Whether the build only previewed changes without writing files.
    pub dry_run: bool,
}

/// Workspace i18n ARB check report.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct I18nCheckReport {
    /// Number of Dart source files scanned.
    pub scanned_files: usize,
    /// Number of unique static translation keys found.
    pub keys: usize,
    /// Number of configured ARB files inspected.
    pub arb_files: usize,
    /// Number of configured locale messages checked against the static scan.
    pub checked_messages: usize,
    /// Number of existing ARB messages absent from the static scan.
    pub stale_messages: usize,
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
    /// The Dust CLI version that owns the active compatibility contract.
    pub cli_version: String,
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
    /// Dust runtime package compatibility rows.
    pub package_compatibility: Vec<DoctorPackageCompatibility>,
}

/// One Dust runtime package compatibility row for `dust doctor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorPackageCompatibility {
    /// Dart package name.
    pub package_name: String,
    /// Whether workspace source imports this package.
    pub used_by_workspace: bool,
    /// Resolved package version from package_config, when present.
    pub resolved_version: Option<String>,
    /// Supported package version constraint for this CLI, when known.
    pub supported_constraint: Option<String>,
    /// Compatibility status for this package.
    pub status: DoctorPackageCompatibilityStatus,
    /// Suggested user action when the status is not healthy.
    pub action: Option<String>,
}

/// Compatibility status for one Dust runtime package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorPackageCompatibilityStatus {
    /// Package is resolved and satisfies the CLI compatibility contract.
    Compatible,
    /// Workspace source imports the package but package_config does not resolve it.
    Missing,
    /// Package is not resolved and is not used by workspace source.
    NotResolved,
    /// Package is older than the supported range.
    TooOld,
    /// Package is newer than the supported range.
    TooNew,
    /// The embedded compatibility contract has no rule for this package.
    UnknownRule,
}

impl DoctorPackageCompatibilityStatus {
    /// Returns the stable lowercase CLI rendering for this status.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compatible => "compatible",
            Self::Missing => "missing",
            Self::NotResolved => "not-resolved",
            Self::TooOld => "too-old",
            Self::TooNew => "too-new",
            Self::UnknownRule => "unknown-rule",
        }
    }
}

/// One clean command summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanReport {
    /// The detected package root.
    pub package_root: PathBuf,
    /// The number of generated Dust-owned files inspected.
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

/// Route table inspection report.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteTableReport {
    /// Number of Dart source libraries scanned.
    pub scanned_files: usize,
    /// Deterministic route inspection entries.
    pub routes: Vec<RouteInspectionEntry>,
}

/// One route inspection entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteInspectionEntry {
    /// Effective route name.
    pub name: String,
    /// Absolute route path.
    pub path: String,
    /// Flutter page class.
    pub page: String,
    /// Effective shell widget class, or absent.
    pub shell: Option<String>,
    /// Effective branch name, or absent.
    pub branch: Option<String>,
    /// Guard class names applied directly to this route.
    pub guards: Vec<String>,
    /// Whether generated code treats this route as auth-protected.
    pub requires_auth: bool,
    /// Route push result type.
    pub result_type: String,
}
