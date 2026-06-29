use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;

use crate::result::I18nScanEntry;

/// One validated scanned key ready for ARB reconciliation or checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct I18nPlannedEntry {
    /// Full Dart translation key.
    pub(crate) key: String,
    /// ARB namespace and asset file stem.
    pub(crate) namespace: String,
    /// Message key inside the namespace ARB file.
    pub(crate) local_key: String,
    /// Optional fallback text from source.
    pub(crate) default_text: Option<String>,
    /// Placeholder names from source.
    pub(crate) args: Vec<String>,
}

/// Validates and converts scanned entries into planned i18n entries.
pub(crate) fn plan_i18n_entries(
    entries: &[I18nScanEntry],
) -> Result<Vec<I18nPlannedEntry>, Diagnostic> {
    let mut planned = Vec::with_capacity(entries.len());
    for entry in entries {
        planned.push(plan_i18n_entry(entry)?);
    }
    Ok(planned)
}

/// Groups planned entries by namespace in deterministic order.
pub(crate) fn group_i18n_entries(
    entries: &[I18nPlannedEntry],
) -> BTreeMap<String, Vec<I18nPlannedEntry>> {
    let mut grouped = BTreeMap::<String, Vec<I18nPlannedEntry>>::new();
    for entry in entries {
        grouped
            .entry(entry.namespace.clone())
            .or_default()
            .push(entry.clone());
    }
    grouped
}

/// Returns the default ARB path for one locale and namespace.
pub(crate) fn i18n_arb_path(package_root: &Path, locale: &str, namespace: &str) -> PathBuf {
    package_root
        .join("assets/i18n")
        .join(locale)
        .join(format!("{namespace}.arb"))
}

/// Returns whether text is an ARB-safe identifier.
pub(crate) fn is_arb_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic() && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Validates one scanned i18n entry.
fn plan_i18n_entry(entry: &I18nScanEntry) -> Result<I18nPlannedEntry, Diagnostic> {
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

    Ok(I18nPlannedEntry {
        key: entry.key.clone(),
        namespace: entry.namespace.clone(),
        local_key,
        default_text: entry.default_text.clone(),
        args: entry.args.clone(),
    })
}

/// Builds an invalid i18n key diagnostic.
fn invalid_key(key: &str, reason: &str) -> Diagnostic {
    Diagnostic::error(format!("invalid i18n key `{key}`: {reason}"))
}
