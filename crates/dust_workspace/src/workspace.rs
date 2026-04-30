use std::path::{Path, PathBuf};

use dust_diagnostics::Diagnostic;

use crate::{PackageConfig, detect_workspace_root, discover_libraries, load_package_config};

/// One source library selected for Dust processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLibrary {
    /// The source `.dart` file path.
    pub source_path: PathBuf,
    /// The generated `.g.dart` output path derived from the source file.
    pub output_path: PathBuf,
}

/// The discovered workspace state needed by later pipeline phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePlan {
    /// The detected Dart package root used for library discovery.
    pub package_root: PathBuf,
    /// The package-local cache root used for `.dart_tool/dust`.
    pub cache_root: PathBuf,
    /// The loaded package configuration.
    pub package_config: PackageConfig,
    /// Candidate Dust source libraries in deterministic order.
    pub libraries: Vec<SourceLibrary>,
}

/// Discovers the workspace root, package configuration, and candidate source libraries.
pub fn discover_workspace(cwd: &Path) -> Result<WorkspacePlan, Diagnostic> {
    let package_root = detect_workspace_root(cwd)?;
    let package_config = load_package_config(&package_root)?;
    let libraries = discover_libraries(&package_root)?;

    Ok(WorkspacePlan {
        cache_root: package_root.clone(),
        package_root,
        package_config,
        libraries,
    })
}
