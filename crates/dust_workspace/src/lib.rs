#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Workspace discovery and Dart library candidate scanning for Dust."]

mod config;
mod discover;
mod output_policy;
mod package_config;
mod pubspec;
mod root;
mod workspace;

pub use config::{DustConfig, OutputConfig, load_dust_config};
pub use discover::discover_libraries;
pub use output_policy::{
    expected_part_uri, generated_test_output_path, is_generated_primary_file, package_import_uri,
    primary_output_path, rewrite_library_import_uri,
};
pub use package_config::{PackageConfig, PackageConfigKind, load_package_config};
pub use pubspec::load_package_name;
pub use root::detect_workspace_root;
pub use workspace::{SourceLibrary, WorkspacePlan, discover_workspace};
