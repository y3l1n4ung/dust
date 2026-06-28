#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Workspace discovery and Dart library candidate scanning for Dust."]

/// Dust workspace configuration.
mod config;
/// Source library discovery.
mod discover;
/// Generated output path policy.
mod output_policy;
/// Dart package config discovery.
mod package_config;
/// Pubspec parsing.
mod pubspec;
/// Workspace root detection.
mod root;
/// Workspace planning.
mod workspace;

pub use config::{DustConfig, I18nConfig, OutputConfig, load_dust_config};
pub use discover::{SupportedAnnotations, discover_libraries};
pub use output_policy::{
    expected_part_uri, generated_test_output_path, is_generated_primary_file, package_import_uri,
    primary_output_path, rewrite_library_import_uri,
};
pub use package_config::{PackageConfig, PackageConfigKind, load_package_config};
pub use pubspec::load_package_name;
pub use root::detect_workspace_root;
pub use workspace::{SourceLibrary, WorkspacePlan, discover_workspace};
