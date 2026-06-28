use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use serde::Deserialize;

/// Default suffix for primary generated Dart libraries.
const DEFAULT_PRIMARY_SUFFIX: &str = ".g.dart";

/// Workspace-level Dust configuration loaded from `dust.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DustConfig {
    /// The resolved config file path when `dust.yaml` exists.
    pub path: Option<PathBuf>,
    /// Generated output policy settings.
    pub outputs: OutputConfig,
    /// Optional i18n generation settings.
    pub i18n: Option<I18nConfig>,
}

/// Generated output policy settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputConfig {
    /// Primary generated library suffix, for example `.g.dart` or `.d.dart`.
    pub primary_suffix: String,
}

/// i18n generation settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18nConfig {
    /// Locale codes supported by generated i18n bootstrap.
    pub locales: Vec<String>,
}

impl I18nConfig {
    /// Returns the fallback locale, which is always the first configured locale.
    pub fn fallback_locale(&self) -> &str {
        self.locales.first().map_or("", String::as_str)
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            primary_suffix: DEFAULT_PRIMARY_SUFFIX.to_owned(),
        }
    }
}

/// Deserialized root object from `dust.yaml`.
#[derive(Debug, Default, Deserialize)]
struct RawDustConfig {
    /// Raw output policy section.
    outputs: Option<RawOutputConfig>,
    /// Raw i18n section.
    i18n: Option<RawI18nConfig>,
}

/// Deserialized `outputs` section from `dust.yaml`.
#[derive(Debug, Default, Deserialize)]
struct RawOutputConfig {
    /// Optional primary generated library suffix.
    primary_suffix: Option<String>,
}

/// Deserialized `i18n` section from `dust.yaml`.
#[derive(Debug, Default, Deserialize)]
struct RawI18nConfig {
    /// Locale codes used by generated i18n bootstrap.
    locales: Option<Vec<String>>,
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
    let i18n = parse_i18n_config(raw.i18n, &path)?;

    Ok(DustConfig {
        path: Some(path),
        outputs: OutputConfig { primary_suffix },
        i18n,
    })
}

/// Parses and validates optional i18n config.
fn parse_i18n_config(
    raw: Option<RawI18nConfig>,
    path: &Path,
) -> Result<Option<I18nConfig>, Diagnostic> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let Some(locales) = raw.locales else {
        return Err(invalid_config(path, "i18n.locales is required"));
    };
    validate_i18n_locales(&locales, path)?;
    Ok(Some(I18nConfig { locales }))
}

/// Validates configured i18n locale codes.
fn validate_i18n_locales(locales: &[String], path: &Path) -> Result<(), Diagnostic> {
    if locales.is_empty() {
        return Err(invalid_config(path, "i18n.locales must not be empty"));
    }

    let mut seen = HashSet::new();
    for locale in locales {
        if locale.is_empty() {
            return Err(invalid_config(
                path,
                "i18n.locales must not contain empty values",
            ));
        }
        if locale.contains('/') || locale.contains('\\') {
            return Err(invalid_config(
                path,
                "i18n.locales must contain locale codes, not paths",
            ));
        }
        if !seen.insert(locale) {
            return Err(invalid_config(
                path,
                "i18n.locales must not contain duplicate values",
            ));
        }
    }

    Ok(())
}

/// Validates the configured generated library suffix.
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

/// Builds an invalid suffix diagnostic.
fn invalid_suffix(path: &Path, message: &str) -> Diagnostic {
    invalid_config(path, message)
}

/// Builds an invalid Dust configuration diagnostic.
fn invalid_config(path: &Path, message: &str) -> Diagnostic {
    Diagnostic::error(format!(
        "invalid Dust configuration `{}`: {message}",
        path.display()
    ))
}
