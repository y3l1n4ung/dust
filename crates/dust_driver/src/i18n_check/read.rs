use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use dust_workspace::I18nConfig;
use serde_json::{Map, Value};

use crate::{
    i18n_keys::{I18nPlannedEntry, i18n_arb_path},
    result::I18nCheckReport,
};

/// Parsed ARB file object.
pub(super) struct ArbFile {
    /// Top-level ARB object.
    pub(super) map: Map<String, Value>,
}

/// One configured ARB file plus optional parsed contents.
pub(super) struct CheckedArbFile {
    /// Locale code associated with the configured path.
    pub(super) locale: String,
    /// Absolute ARB file path.
    pub(super) path: PathBuf,
    /// Parsed ARB object when available.
    pub(super) arb: Option<ArbFile>,
}

/// Returns scanned and existing ARB namespaces that should be checked.
pub(super) fn namespaces_to_check(
    package_root: &Path,
    config: &I18nConfig,
    by_namespace: &BTreeMap<String, Vec<I18nPlannedEntry>>,
) -> Result<BTreeSet<String>, Diagnostic> {
    let mut namespaces = by_namespace.keys().cloned().collect::<BTreeSet<_>>();
    for locale in &config.locales {
        collect_existing_namespaces(package_root, locale, &mut namespaces)?;
    }
    Ok(namespaces)
}

/// Reads all configured locale files for one namespace.
pub(super) fn read_namespace_files(
    package_root: &Path,
    config: &I18nConfig,
    namespace: &str,
    report: &mut I18nCheckReport,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<CheckedArbFile> {
    let mut files = Vec::with_capacity(config.locales.len());
    for locale in &config.locales {
        report.arb_files += 1;
        let path = i18n_arb_path(package_root, locale, namespace);
        let arb = read_arb_file(locale, &path, diagnostics);
        files.push(CheckedArbFile {
            locale: locale.clone(),
            path,
            arb,
        });
    }
    files
}

/// Returns fallback-locale message strings by local ARB key.
pub(super) fn fallback_messages(
    config: &I18nConfig,
    files: &[CheckedArbFile],
) -> Option<BTreeMap<String, String>> {
    let fallback = config.fallback_locale();
    let file = files
        .iter()
        .find(|file| file.locale == fallback)
        .and_then(|file| file.arb.as_ref())?;
    let mut messages = BTreeMap::new();
    for (key, value) in &file.map {
        if !is_message_key(key) {
            continue;
        }
        if let Some(message) = value.as_str() {
            messages.insert(key.clone(), message.to_owned());
        }
    }
    Some(messages)
}

/// Adds existing ARB namespace file stems for one locale.
fn collect_existing_namespaces(
    package_root: &Path,
    locale: &str,
    namespaces: &mut BTreeSet<String>,
) -> Result<(), Diagnostic> {
    let dir = package_root.join("assets/i18n").join(locale);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(Diagnostic::error(format!(
                "failed to read i18n asset directory `{}`: {error}",
                dir.display()
            )));
        }
    };

    for entry in entries {
        let entry = entry.map_err(|error| {
            Diagnostic::error(format!(
                "failed to enumerate i18n asset directory `{}`: {error}",
                dir.display()
            ))
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("arb") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            namespaces.insert(stem.to_owned());
        }
    }
    Ok(())
}

/// Reads and parses one ARB file.
fn read_arb_file(locale: &str, path: &Path, diagnostics: &mut Vec<Diagnostic>) -> Option<ArbFile> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            diagnostics.push(Diagnostic::error(format!(
                "missing i18n ARB file `{}` for locale `{locale}`",
                path.display()
            )));
            return None;
        }
        Err(error) => {
            diagnostics.push(Diagnostic::error(format!(
                "failed to read i18n ARB file `{}`: {error}",
                path.display()
            )));
            return None;
        }
    };

    let value = match serde_json::from_str::<Value>(&source) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(Diagnostic::error(format!(
                "failed to parse i18n ARB file `{}`: {error}",
                path.display()
            )));
            return None;
        }
    };
    match value {
        Value::Object(map) => Some(ArbFile { map }),
        _ => {
            diagnostics.push(Diagnostic::error(format!(
                "i18n ARB file `{}` must contain a JSON object",
                path.display()
            )));
            None
        }
    }
}

/// Returns whether one ARB top-level key is a message key.
fn is_message_key(key: &str) -> bool {
    !key.starts_with('@')
}
