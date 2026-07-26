use std::{collections::BTreeMap, fs, path::Path};

use dust_dart_syntax::DartLanguageVersion;
use dust_diagnostics::Diagnostic;
use serde::Deserialize;

/// Minimal pubspec fields needed by workspace discovery.
#[derive(Debug, Deserialize)]
struct Pubspec {
    /// Optional package name from `pubspec.yaml`.
    name: Option<String>,
    /// Package dependencies.
    #[serde(default)]
    dependencies: BTreeMap<String, serde_yaml::Value>,
    /// Package development dependencies.
    #[serde(default)]
    dev_dependencies: BTreeMap<String, serde_yaml::Value>,
    /// Dart environment constraints.
    #[serde(default)]
    environment: BTreeMap<String, serde_yaml::Value>,
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

/// Returns whether `pubspec.yaml` describes a Flutter package.
pub fn load_is_flutter_package(package_root: &Path) -> Result<bool, Diagnostic> {
    let path = package_root.join("pubspec.yaml");
    let parsed = parse_pubspec(&path)?;
    Ok(parsed.flutter.is_some()
        || parsed.dependencies.contains_key("flutter")
        || parsed.dev_dependencies.contains_key("flutter"))
}

/// Loads the lower Dart SDK language version from `pubspec.yaml`, if declared.
pub fn load_dart_sdk_lower_bound(
    package_root: &Path,
) -> Result<Option<DartLanguageVersion>, Diagnostic> {
    let path = package_root.join("pubspec.yaml");
    let parsed = parse_pubspec(&path)?;
    let Some(value) = parsed.environment.get("sdk") else {
        return Ok(None);
    };
    let Some(constraint) = pubspec_string_value(value) else {
        return Ok(None);
    };
    Ok(DartLanguageVersion::from_sdk_constraint_lower_bound(
        constraint,
    ))
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

/// Converts a scalar pubspec value into a string slice when supported.
fn pubspec_string_value(value: &serde_yaml::Value) -> Option<&str> {
    match value {
        serde_yaml::Value::String(value) => Some(value.as_str()),
        _ => None,
    }
}
