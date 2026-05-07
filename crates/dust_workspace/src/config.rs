use std::{
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use serde::Deserialize;

const DEFAULT_PRIMARY_SUFFIX: &str = ".g.dart";

/// Workspace-level Dust configuration loaded from `dust.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DustConfig {
    /// The resolved config file path when `dust.yaml` exists.
    pub path: Option<PathBuf>,
    /// Generated output policy settings.
    pub outputs: OutputConfig,
}

/// Generated output policy settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputConfig {
    /// Primary generated library suffix, for example `.g.dart` or `.d.dart`.
    pub primary_suffix: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            primary_suffix: DEFAULT_PRIMARY_SUFFIX.to_owned(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawDustConfig {
    outputs: Option<RawOutputConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct RawOutputConfig {
    primary_suffix: Option<String>,
}

/// Loads `dust.yaml` when present, otherwise returns the default output policy.
pub fn load_dust_config(package_root: &Path) -> Result<DustConfig, Diagnostic> {
    let path = package_root.join("dust.yaml");
    if !path.is_file() {
        return Ok(DustConfig::default());
    }

    let source = fs::read_to_string(&path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read Dust configuration `{}`: {error}",
            path.display()
        ))
    })?;
    let raw = serde_yaml::from_str::<RawDustConfig>(&source).map_err(|error| {
        Diagnostic::error(format!(
            "failed to parse Dust configuration `{}`: {error}",
            path.display()
        ))
    })?;

    let primary_suffix = raw
        .outputs
        .and_then(|outputs| outputs.primary_suffix)
        .unwrap_or_else(|| DEFAULT_PRIMARY_SUFFIX.to_owned());
    validate_primary_suffix(&primary_suffix, &path)?;

    Ok(DustConfig {
        path: Some(path),
        outputs: OutputConfig { primary_suffix },
    })
}

fn validate_primary_suffix(primary_suffix: &str, path: &Path) -> Result<(), Diagnostic> {
    if primary_suffix.is_empty() {
        return Err(invalid_suffix(
            path,
            "outputs.primary_suffix must not be empty",
        ));
    }
    if primary_suffix.contains('/') || primary_suffix.contains('\\') {
        return Err(invalid_suffix(
            path,
            "outputs.primary_suffix must be a file suffix, not a path",
        ));
    }
    if !primary_suffix.starts_with('.') || !primary_suffix.ends_with(".dart") {
        return Err(invalid_suffix(
            path,
            "outputs.primary_suffix must start with `.` and end with `.dart`",
        ));
    }
    Ok(())
}

fn invalid_suffix(path: &Path, message: &str) -> Diagnostic {
    Diagnostic::error(format!(
        "invalid Dust configuration `{}`: {message}",
        path.display()
    ))
}
