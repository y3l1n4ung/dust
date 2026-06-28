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
/// Shared workspace, registry, and cache loading context.
mod context;
/// Doctor command diagnostics for workspace setup.
mod doctor;
/// Conversion from resolved parser data into Dust IR.
mod lower;
/// Public progress event model.
mod progress;
/// Public command request model.
mod request;
/// Public command result model.
mod result;
/// Watch command orchestration and change snapshots.
mod watch;

pub use build::{run_build, run_build_with_progress};
pub use check::run_check;
pub use clean::run_clean;
pub use doctor::run_doctor;
pub use dust_parser_dart_ts::{
    I18nScanResult, I18nTranslationKind, I18nTranslationUse, scan_i18n_source,
};
pub use progress::{ProgressEvent, ProgressPhase};
pub use request::{
    BuildRequest, CheckRequest, CleanRequest, CommandRequest, DbRequestOptions, DoctorRequest,
    WatchRequest,
};
pub use result::{
    BuildArtifact, CacheReport, CheckedLibrary, CleanReport, CommandResult, DiagnosticFile,
    DoctorReport, WatchReport,
};
pub use watch::{run_watch, run_watch_with_progress};

/// Runs one driver command request.
pub fn run(request: CommandRequest) -> CommandResult {
    match request {
        CommandRequest::Build(request) => run_build(request),
        CommandRequest::Clean(request) => run_clean(request),
        CommandRequest::Check(request) => run_check(request),
        CommandRequest::Doctor(request) => run_doctor(request),
        CommandRequest::Watch(request) => run_watch(request),
    }
}
