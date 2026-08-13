use std::path::PathBuf;

/// Database execution options for build/check commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DbRequestOptions {
    /// Whether only Database generation/validation should run.
    pub only_db: bool,
    /// Whether Database should use offline query metadata only.
    pub offline: bool,
}

/// One build request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
    /// Whether the driver should stop after the first error diagnostic.
    pub fail_fast: bool,
    /// The optional parallel job count for library processing.
    ///
    /// `None` lets the driver choose its default execution policy.
    pub jobs: Option<usize>,
    /// Optional Database focused mode.
    pub db: DbRequestOptions,
}

/// One check request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
    /// Whether the driver should stop after the first error diagnostic.
    pub fail_fast: bool,
    /// The optional parallel job count for library processing.
    ///
    /// `None` lets the driver choose its default execution policy.
    pub jobs: Option<usize>,
    /// Optional Database focused mode.
    pub db: DbRequestOptions,
}

/// One doctor request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
}

/// One route table inspection request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteTableRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
}

/// One route graph inspection request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteGraphRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
}

/// One i18n scan request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18nScanRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
}

/// One i18n build request.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct I18nBuildRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
    /// Whether existing fallback-locale messages may be synced from `defaultText`.
    pub sync_source: bool,
    /// Whether to preview planned sync changes without writing files.
    pub dry_run: bool,
}

/// One i18n check request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18nCheckRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
}

/// One clean request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
}

/// One watch request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchRequest {
    /// The working directory used to discover the Dart workspace.
    pub cwd: PathBuf,
    /// Whether the driver should stop after the first error diagnostic.
    pub fail_fast: bool,
    /// The optional parallel job count for library processing.
    ///
    /// `None` lets the driver choose its default execution policy.
    pub jobs: Option<usize>,
    /// The number of milliseconds to wait between filesystem polls.
    pub poll_interval_ms: u64,
    /// An optional upper bound on poll cycles, mainly used by tests.
    ///
    /// `None` means watch continuously until the caller stops the process.
    pub max_cycles: Option<u32>,
}

/// One supported driver command request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandRequest {
    /// Run a writing build.
    Build(BuildRequest),
    /// Remove Dust-generated outputs and cache state.
    Clean(CleanRequest),
    /// Run a no-write freshness check.
    Check(CheckRequest),
    /// Report workspace and plugin readiness.
    Doctor(DoctorRequest),
    /// Print the generated app route table.
    RouteTable(RouteTableRequest),
    /// Print the generated app route graph.
    RouteGraph(RouteGraphRequest),
    /// Reconcile scanned i18n keys into ARB assets.
    I18nBuild(I18nBuildRequest),
    /// Validate configured i18n ARB assets without writing files.
    I18nCheck(I18nCheckRequest),
    /// Scan static i18n API calls.
    I18nScan(I18nScanRequest),
    /// Run initial build plus repeated rebuild polling.
    Watch(WatchRequest),
}
