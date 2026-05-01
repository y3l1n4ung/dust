#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "End-to-end build orchestration for Dust workspace generation."]

mod build;
mod catalog;
mod check;
mod clean;
mod context;
mod doctor;
mod lower;
mod progress;
mod request;
mod result;
mod watch;

pub use build::{run_build, run_build_with_progress};
pub use check::run_check;
pub use clean::run_clean;
pub use doctor::run_doctor;
pub use progress::{ProgressEvent, ProgressPhase};
pub use request::{
    BuildRequest, CheckRequest, CleanRequest, CommandRequest, DoctorRequest, WatchRequest,
};
pub use result::{
    BuildArtifact, CacheReport, CheckedLibrary, CleanReport, CommandResult, DoctorReport,
    WatchReport,
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
