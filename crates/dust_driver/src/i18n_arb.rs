use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use dust_workspace::I18nConfig;
use serde_json::{Map, Value};

/// Deterministic ARB JSON rendering.
mod render;
/// Atomic file writing helpers.
mod write;

use self::{render::render_arb, write::write_atomic};
use crate::result::{I18nBuildReport, I18nScanEntry};

/// Reconciles scanned i18n entries into configured ARB files.
pub(crate) fn write_i18n_arb_files(
    package_root: &Path,
    config: &I18nConfig,
    scanned_files: usize,
    entries: &[I18nScanEntry],
) -> Result<I18nBuildReport, Diagnostic> {
    let planned = plan_entries(entries)?;
    let grouped = group_entries(&planned);
    let mut report = I18nBuildReport {
        scanned_files,
        keys: planned.len(),
        ..I18nBuildReport::default()
    };
    let updates = plan_updates(package_root, config, &grouped, &mut report)?;

    for update in updates {
        write_atomic(&update.path, &update.source).map_err(|error| {
            Diagnostic::error(format!(
                "failed to write `{}`: {error}",
                update.path.display()
            ))
        })?;
    }

    Ok(report)
}

/// One validated scanned key ready for ARB reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PlannedEntry {
    /// Full Dart translation key.
    key: String,
    /// ARB namespace and asset file stem.
    namespace: String,
    /// Message key inside the namespace ARB file.
    local_key: String,
    /// Optional fallback text from source.
    default_text: Option<String>,
    /// Placeholder names from source.
    args: Vec<String>,
}

/// One ARB file update prepared before any writes happen.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ArbUpdate {
    /// ARB file path to write.
    path: PathBuf,
    /// Rendered ARB JSON source.
    source: String,
}

/// Validates and converts scanned entries into writer entries.
fn plan_entries(entries: &[I18nScanEntry]) -> Result<Vec<PlannedEntry>, Diagnostic> {
    let mut planned = Vec::with_capacity(entries.len());
    for entry in entries {
        planned.push(plan_entry(entry)?);
    }
    Ok(planned)
}

/// Validates one scanned entry.
fn plan_entry(entry: &I18nScanEntry) -> Result<PlannedEntry, Diagnostic> {
    if entry.namespace.is_empty() {
        return Err(invalid_key(&entry.key, "missing namespace prefix"));
    }
    let prefix = format!("{}_", entry.namespace);
    if !entry.key.starts_with(&prefix) {
        return Err(invalid_key(
            &entry.key,
            "use an underscore namespace prefix such as `shop_title`",
        ));
    }
    let local_key = entry.key[prefix.len()..].to_owned();
    if !is_arb_identifier(&entry.namespace) || !is_arb_identifier(&local_key) {
        return Err(invalid_key(
            &entry.key,
            "keys must start with a letter and contain only letters, numbers, or underscores",
        ));
    }

    Ok(PlannedEntry {
        key: entry.key.clone(),
        namespace: entry.namespace.clone(),
        local_key,
        default_text: entry.default_text.clone(),
        args: entry.args.clone(),
    })
}

/// Groups entries by namespace in deterministic order.
fn group_entries(entries: &[PlannedEntry]) -> BTreeMap<String, Vec<PlannedEntry>> {
    let mut grouped = BTreeMap::<String, Vec<PlannedEntry>>::new();
    for entry in entries {
        grouped
            .entry(entry.namespace.clone())
            .or_default()
            .push(entry.clone());
    }
    grouped
}

/// Builds all file updates before writing any file.
fn plan_updates(
    package_root: &Path,
    config: &I18nConfig,
    grouped: &BTreeMap<String, Vec<PlannedEntry>>,
    report: &mut I18nBuildReport,
) -> Result<Vec<ArbUpdate>, Diagnostic> {
    let mut updates = Vec::new();
    for (namespace, entries) in grouped {
        for locale in &config.locales {
            report.arb_files += 1;
            let path = arb_path(package_root, locale, namespace);
            if let Some(update) = plan_file_update(
                &path,
                locale,
                locale == config.fallback_locale(),
                entries,
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
    locale: &str,
    is_fallback: bool,
    entries: &[PlannedEntry],
    report: &mut I18nBuildReport,
) -> Result<Option<ArbUpdate>, Diagnostic> {
    let previous = read_optional(path)?;
    let mut arb = parse_arb(path, previous.as_deref())?;
    let mut changed = ensure_locale(&mut arb, locale);

    for entry in entries {
        let added = ensure_message(&mut arb, entry, is_fallback, path)?;
        changed |= added;
        if added {
            report.added_messages += 1;
        }
        changed |= ensure_metadata(&mut arb, entry, added);
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

/// Ensures one message entry exists.
fn ensure_message(
    arb: &mut Map<String, Value>,
    entry: &PlannedEntry,
    is_fallback: bool,
    path: &Path,
) -> Result<bool, Diagnostic> {
    if let Some(existing) = arb.get(&entry.local_key) {
        if existing.is_string() {
            return Ok(false);
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
    Ok(true)
}

/// Ensures metadata exists for new keys and placeholder-bearing existing keys.
fn ensure_metadata(arb: &mut Map<String, Value>, entry: &PlannedEntry, added: bool) -> bool {
    let metadata_key = format!("@{}", entry.local_key);
    if arb.contains_key(&metadata_key) || (!added && entry.args.is_empty()) {
        return false;
    }
    arb.insert(metadata_key, metadata_value(&entry.args));
    true
}

/// Builds ARB metadata for one message.
fn metadata_value(args: &[String]) -> Value {
    if args.is_empty() {
        return Value::Object(Map::new());
    }

    let mut placeholders = Map::new();
    for arg in args {
        placeholders.insert(arg.clone(), Value::Object(Map::new()));
    }
    let mut metadata = Map::new();
    metadata.insert("placeholders".to_owned(), Value::Object(placeholders));
    Value::Object(metadata)
}

/// Returns the default ARB path for one locale and namespace.
fn arb_path(package_root: &Path, locale: &str, namespace: &str) -> PathBuf {
    package_root
        .join("assets/i18n")
        .join(locale)
        .join(format!("{namespace}.arb"))
}

/// Returns whether text is an ARB-safe identifier.
fn is_arb_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic() && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Builds an invalid i18n key diagnostic.
fn invalid_key(key: &str, reason: &str) -> Diagnostic {
    Diagnostic::error(format!("invalid i18n key `{key}`: {reason}"))
}
