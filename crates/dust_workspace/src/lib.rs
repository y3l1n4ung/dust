#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Workspace discovery and Dart library candidate scanning for Dust."]

mod discover;
mod package_config;
mod root;
mod workspace;

pub use discover::discover_libraries;
pub use package_config::{PackageConfig, load_package_config};
pub use root::detect_workspace_root;
pub use workspace::{SourceLibrary, WorkspacePlan, discover_workspace};
