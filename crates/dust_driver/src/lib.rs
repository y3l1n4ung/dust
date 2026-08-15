#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "End-to-end build orchestration for Dust workspace generation."]

/// Build command orchestration and file emission.
mod build;
/// Resolver symbol catalog construction.
mod catalog;
/// Check command orchestration without writing generated files.
mod check;
/// Clean command support for generated files and cache state.
mod clean;
/// Dust CLI and Dart package compatibility checks.
mod compatibility;
/// Shared workspace, registry, and cache loading context.
mod context;
/// Doctor command diagnostics for workspace setup.
mod doctor;
/// ARB reconciliation helpers for i18n build.
mod i18n_arb;
/// Flutter asset declaration checks for i18n ARB files.
mod i18n_assets;
/// Generated i18n bootstrap output.
mod i18n_bootstrap;
/// Workspace i18n ARB build command.
mod i18n_build;
/// Workspace i18n ARB validation command.
mod i18n_check;
/// Native iOS locale metadata integration.
mod i18n_ios;
/// Shared i18n key planning helpers.
mod i18n_keys;
/// Workspace i18n source scanning.
mod i18n_scan;
/// Conversion from resolved parser data into Dust IR.
mod lower;
/// Public progress event model.
mod progress;
/// Public command request model.
mod request;
/// Public command result model.
mod result;
/// Route deep-link fixture inspection command support.
/// Route graph inspection command support.
/// Shared route inspection workspace scanning.
mod route_inspection;
/// Route inspection command support.
mod route_table;
/// Watch command orchestration and change snapshots.
mod watch;

pub use build::{run_build, run_build_with_progress};
pub use check::run_check;
pub use clean::run_clean;
pub use doctor::run_doctor;
pub use dust_parser_dart_ts::{
    I18nScanResult, I18nTranslationKind, I18nTranslationUse, scan_i18n_source,
};
pub use i18n_build::run_i18n_build;
pub use i18n_check::run_i18n_check;
pub use i18n_scan::run_i18n_scan;
pub use progress::{ProgressEvent, ProgressPhase};
pub use request::{
    BuildRequest, CheckRequest, CleanRequest, CommandRequest, DbRequestOptions, DoctorRequest,
    I18nBuildRequest, I18nCheckRequest, I18nScanRequest, RouteTableRequest, WatchRequest,
};
pub use result::{
    BuildArtifact, CacheReport, CheckedLibrary, CleanReport, CommandResult, DiagnosticFile,
    DoctorPackageCompatibility, DoctorPackageCompatibilityStatus, DoctorReport, I18nBuildReport,
    I18nCheckReport, I18nScanEntry, I18nScanReport, RouteInspectionEntry, RouteTableReport,
    WatchReport,
};
pub use route_table::run_route_table;
pub use watch::{run_watch, run_watch_with_progress};

/// Runs one driver command request.
pub fn run(request: CommandRequest) -> CommandResult {
    match request {
        CommandRequest::Build(request) => run_build(request),
        CommandRequest::Clean(request) => run_clean(request),
        CommandRequest::Check(request) => run_check(request),
        CommandRequest::Doctor(request) => run_doctor(request),
        CommandRequest::RouteTable(request) => run_route_table(request),
        CommandRequest::I18nBuild(request) => run_i18n_build(request),
        CommandRequest::I18nCheck(request) => run_i18n_check(request),
        CommandRequest::I18nScan(request) => run_i18n_scan(request),
        CommandRequest::Watch(request) => run_watch(request),
    }
}
