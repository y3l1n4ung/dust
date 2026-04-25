use std::path::{Path, PathBuf};

use dust_diagnostics::Diagnostic;

/// The discovered package configuration for one Dart workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageConfig {
    /// The absolute path to `.dart_tool/package_config.json`.
    pub path: PathBuf,
}

/// Loads the package configuration path for the workspace root.
pub fn load_package_config(root: &Path) -> Result<PackageConfig, Diagnostic> {
    let path = root.join(".dart_tool/package_config.json");
    if !path.is_file() {
        return Err(Diagnostic::error(format!(
            "missing package configuration at `{}`",
            path.display()
        )));
    }

    Ok(PackageConfig { path })
}
