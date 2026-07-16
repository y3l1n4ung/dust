use std::{
    fs, io,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use dust_workspace::I18nConfig;
use serde_json::{Map, Value};

/// ARB message metadata enrichment.
mod metadata;
/// Deterministic ARB JSON rendering.
mod render;
/// Atomic file writing helpers.
mod write;

use self::{metadata::ensure_metadata, render::render_arb, write::write_atomic};
use crate::i18n_keys::{I18nPlannedEntry, group_i18n_entries, i18n_arb_path, plan_i18n_entries};
use crate::result::{I18nBuildReport, I18nScanEntry};

/// Reconciles scanned i18n entries into configured ARB files.
pub(crate) fn write_i18n_arb_files(
    package_root: &Path,
    config: &I18nConfig,
    scanned_files: usize,
    entries: &[I18nScanEntry],
    sync_source: bool,
    dry_run: bool,
) -> Result<I18nBuildReport, Diagnostic> {
    let planned = plan_i18n_entries(entries)?;
    let grouped = group_i18n_entries(&planned);
    let mut report = I18nBuildReport {
        scanned_files,
        keys: planned.len(),
        dry_run,
        ..I18nBuildReport::default()
    };
    let updates = plan_updates(package_root, config, &grouped, sync_source, &mut report)?;

    if !dry_run {
        for update in updates {
            write_atomic(&update.path, &update.source).map_err(|error| {
                Diagnostic::error(format!(
                    "failed to write `{}`: {error}",
                    update.path.display()
                ))
            })?;
        }
    }

    Ok(report)
}

/// One ARB file update prepared before any writes happen.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ArbUpdate {
    /// ARB file path to write.
    path: PathBuf,
    /// Rendered ARB JSON source.
    source: String,
}

/// Builds all file updates before writing any file.
fn plan_updates(
    package_root: &Path,
    config: &I18nConfig,
    grouped: &std::collections::BTreeMap<String, Vec<I18nPlannedEntry>>,
    sync_source: bool,
    report: &mut I18nBuildReport,
) -> Result<Vec<ArbUpdate>, Diagnostic> {
    let mut updates = Vec::new();
    for (namespace, entries) in grouped {
        for locale in &config.locales {
            report.arb_files += 1;
            let path = i18n_arb_path(package_root, locale, namespace);
            if let Some(update) = plan_file_update(
                &path,
                namespace,
                locale,
                locale == config.fallback_locale(),
                entries,
                sync_source,
                report,
            )? {
                updates.push(update);
            }
        }
    }
    Ok(updates)
}

/// Plans one ARB file update.
fn plan_file_update(
    path: &Path,
    namespace: &str,
    locale: &str,
    is_fallback: bool,
    entries: &[I18nPlannedEntry],
    sync_source: bool,
    report: &mut I18nBuildReport,
) -> Result<Option<ArbUpdate>, Diagnostic> {
    let previous = read_optional(path)?;
    let mut arb = parse_arb(path, previous.as_deref())?;
    let mut changed = ensure_locale(&mut arb, locale);
    changed |= ensure_context(&mut arb, namespace);

    for entry in entries {
        match ensure_message(&mut arb, entry, is_fallback, sync_source, path)? {
            MessageUpdate::Added => {
                changed = true;
                report.added_messages += 1;
            }
            MessageUpdate::Synced => {
                changed = true;
                report.synced_messages += 1;
            }
            MessageUpdate::None => {}
        }
        changed |= ensure_metadata(&mut arb, entry);
    }

    if !changed {
        return Ok(None);
    }

    report.changed_files += 1;
    Ok(Some(ArbUpdate {
        path: path.to_path_buf(),
        source: render_arb(&arb)?,
    }))
}

/// Reads an optional text file.
fn read_optional(path: &Path) -> Result<Option<String>, Diagnostic> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Diagnostic::error(format!(
            "failed to read `{}`: {error}",
            path.display()
        ))),
    }
}

/// Parses an existing ARB file, or returns an empty object for a missing file.
fn parse_arb(path: &Path, source: Option<&str>) -> Result<Map<String, Value>, Diagnostic> {
    let Some(source) = source else {
        return Ok(Map::new());
    };
    let value = serde_json::from_str::<Value>(source).map_err(|error| {
        Diagnostic::error(format!("failed to parse `{}`: {error}", path.display()))
    })?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(Diagnostic::error(format!(
            "`{}` must contain a JSON object",
            path.display()
        ))),
    }
}

/// Ensures the top-level ARB locale marker exists and matches config.
fn ensure_locale(arb: &mut Map<String, Value>, locale: &str) -> bool {
    if arb.get("@@locale").and_then(Value::as_str) == Some(locale) {
        return false;
    }
    arb.insert("@@locale".to_owned(), Value::String(locale.to_owned()));
    true
}

/// Ensures a deterministic namespace-level context exists.
fn ensure_context(arb: &mut Map<String, Value>, namespace: &str) -> bool {
    if arb.contains_key("@@context") {
        return false;
    }
    arb.insert(
        "@@context".to_owned(),
        Value::String(format!("Translations for `{namespace}` namespace.")),
    );
    true
}

/// Ensures one message entry exists.
fn ensure_message(
    arb: &mut Map<String, Value>,
    entry: &I18nPlannedEntry,
    is_fallback: bool,
    sync_source: bool,
    path: &Path,
) -> Result<MessageUpdate, Diagnostic> {
    if let Some(existing) = arb.get(&entry.local_key) {
        if existing.is_string() {
            if sync_source && is_fallback {
                if let Some(default_text) = &entry.default_text {
                    if existing.as_str() != Some(default_text) {
                        arb.insert(entry.local_key.clone(), Value::String(default_text.clone()));
                        return Ok(MessageUpdate::Synced);
                    }
                }
            }
            return Ok(MessageUpdate::None);
        }
        return Err(Diagnostic::error(format!(
            "`{}` key `{}` must be a string",
            path.display(),
            entry.local_key
        )));
    }

    let value = if is_fallback {
        entry
            .default_text
            .clone()
            .unwrap_or_else(|| entry.key.clone())
    } else {
        String::new()
    };
    arb.insert(entry.local_key.clone(), Value::String(value));
    Ok(MessageUpdate::Added)
}

/// Describes how one ARB message changed during reconciliation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageUpdate {
    /// The existing value and metadata were preserved.
    None,
    /// A missing message was added.
    Added,
    /// An existing fallback-locale value was synced from `defaultText`.
    Synced,
}
