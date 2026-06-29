use std::{fs, path::Path};

use dust_diagnostics::Diagnostic;
use serde::Deserialize;

/// Minimal pubspec fields needed by workspace discovery.
#[derive(Debug, Deserialize)]
struct Pubspec {
    /// Optional package name from `pubspec.yaml`.
    name: Option<String>,
    /// Optional Flutter-specific configuration.
    flutter: Option<FlutterPubspec>,
}

/// Flutter-specific pubspec fields used by Dust.
#[derive(Debug, Deserialize)]
struct FlutterPubspec {
    /// Declared asset entries.
    #[serde(default)]
    assets: Vec<FlutterAsset>,
}

/// One supported Flutter asset entry shape.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FlutterAsset {
    /// Plain asset path entry.
    Path(String),
    /// Object asset entry with an explicit path.
    Object {
        /// Asset path used by Flutter.
        path: String,
    },
}

/// Loads the package name from `pubspec.yaml`.
pub fn load_package_name(package_root: &Path) -> Result<String, Diagnostic> {
    let path = package_root.join("pubspec.yaml");
    let parsed = parse_pubspec(&path)?;
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

/// Loads Flutter asset declarations from `pubspec.yaml`.
pub fn load_flutter_assets(package_root: &Path) -> Result<Vec<String>, Diagnostic> {
    let path = package_root.join("pubspec.yaml");
    let parsed = parse_pubspec(&path)?;
    let assets = parsed
        .flutter
        .map(|flutter| {
            flutter
                .assets
                .into_iter()
                .map(|asset| match asset {
                    FlutterAsset::Path(path) | FlutterAsset::Object { path } => path,
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(assets)
}

/// Parses `pubspec.yaml`.
fn parse_pubspec(path: &Path) -> Result<Pubspec, Diagnostic> {
    let source = fs::read_to_string(path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read pubspec `{}`: {error}",
            path.display()
        ))
    })?;
    serde_yaml::from_str::<Pubspec>(&source).map_err(|error| {
        Diagnostic::error(format!(
            "failed to parse pubspec `{}`: {error}",
            path.display()
        ))
    })
}
