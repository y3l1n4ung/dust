use std::{fs, path::Path};

use dust_diagnostics::Diagnostic;
use serde::Deserialize;

/// Minimal pubspec fields needed by workspace discovery.
#[derive(Debug, Deserialize)]
struct Pubspec {
    /// Optional package name from `pubspec.yaml`.
    name: Option<String>,
}

/// Loads the package name from `pubspec.yaml`.
pub fn load_package_name(package_root: &Path) -> Result<String, Diagnostic> {
    let path = package_root.join("pubspec.yaml");
    let source = fs::read_to_string(&path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read pubspec `{}`: {error}",
            path.display()
        ))
    })?;
    let parsed = serde_yaml::from_str::<Pubspec>(&source).map_err(|error| {
        Diagnostic::error(format!(
            "failed to parse pubspec `{}`: {error}",
            path.display()
        ))
    })?;
    let name = parsed.name.unwrap_or_default();
    let name = name.trim();
    if name.is_empty() {
        return Err(Diagnostic::error(format!(
            "pubspec `{}` must declare a non-empty package name",
            path.display()
        )));
    }
    Ok(name.to_owned())
}
