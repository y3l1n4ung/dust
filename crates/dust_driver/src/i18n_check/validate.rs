use std::collections::{BTreeMap, BTreeSet};

use dust_diagnostics::Diagnostic;
use dust_workspace::I18nConfig;
use serde_json::{Map, Value};

use super::read::{ArbFile, CheckedArbFile};
use crate::{
    i18n_keys::{I18nPlannedEntry, is_arb_identifier},
    result::I18nCheckReport,
};

/// Returns expected entries keyed by local ARB key.
pub(super) fn expected_entries(entries: &[I18nPlannedEntry]) -> BTreeMap<String, I18nPlannedEntry> {
    entries
        .iter()
        .map(|entry| (entry.local_key.clone(), entry.clone()))
        .collect()
}

/// Validates one ARB locale marker.
pub(super) fn validate_locale_marker(file: &CheckedArbFile, diagnostics: &mut Vec<Diagnostic>) {
    let Some(arb) = &file.arb else {
        return;
    };
    if arb.map.get("@@locale").and_then(Value::as_str) == Some(file.locale.as_str()) {
        return;
    }
    diagnostics.push(Diagnostic::error(format!(
        "i18n ARB file `{}` must declare `@@locale` as `{}`",
        file.path.display(),
        file.locale
    )));
}

/// Validates scanned keys against one locale ARB file.
pub(super) fn validate_expected_messages(
    file: &CheckedArbFile,
    expected: &BTreeMap<String, I18nPlannedEntry>,
    fallback: Option<&BTreeMap<String, String>>,
    config: &I18nConfig,
    report: &mut I18nCheckReport,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(arb) = &file.arb else {
        return;
    };
    for (local_key, entry) in expected {
        report.checked_messages += 1;
        let Some(value) = arb.map.get(local_key) else {
            diagnostics.push(Diagnostic::error(format!(
                "missing i18n key `{local_key}` in `{}` for scanned key `{}`",
                file.path.display(),
                entry.key
            )));
            continue;
        };
        let Some(message) = value.as_str() else {
            diagnostics.push(Diagnostic::error(format!(
                "i18n key `{local_key}` in `{}` must be a string",
                file.path.display()
            )));
            continue;
        };
        validate_message_text(file, local_key, message, fallback, config, diagnostics);
        validate_placeholders(file, local_key, entry, message, diagnostics);
    }
}

/// Diagnoses ARB message keys that are absent from the current static scan.
pub(super) fn validate_stale_messages(
    namespace: &str,
    file: &CheckedArbFile,
    expected: &BTreeMap<String, I18nPlannedEntry>,
    report: &mut I18nCheckReport,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(arb) = &file.arb else {
        return;
    };
    for key in arb.map.keys().filter(|key| is_message_key(key)) {
        if expected.contains_key(key) {
            continue;
        }
        report.stale_messages += 1;
        diagnostics.push(Diagnostic::warning(format!(
            "stale i18n key `{namespace}_{key}` in `{}` is not used by current static scan",
            file.path.display()
        )));
    }
}

/// Validates one ARB message string.
fn validate_message_text(
    file: &CheckedArbFile,
    local_key: &str,
    message: &str,
    fallback: Option<&BTreeMap<String, String>>,
    config: &I18nConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if message.is_empty() {
        diagnostics.push(Diagnostic::error(format!(
            "i18n key `{local_key}` in `{}` must not be empty",
            file.path.display()
        )));
        return;
    }
    if file.locale == config.fallback_locale() {
        return;
    }
    let Some(fallback_message) = fallback.and_then(|messages| messages.get(local_key)) else {
        return;
    };
    if message == fallback_message {
        diagnostics.push(Diagnostic::warning(format!(
            "i18n key `{local_key}` in `{}` matches fallback locale `{}`",
            file.path.display(),
            config.fallback_locale()
        )));
    }
}

/// Validates placeholder metadata and message text for one scanned entry.
fn validate_placeholders(
    file: &CheckedArbFile,
    local_key: &str,
    entry: &I18nPlannedEntry,
    message: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let expected = entry.args.iter().cloned().collect::<BTreeSet<_>>();
    let Some(metadata) = message_metadata(file.arb.as_ref(), local_key) else {
        diagnostics.push(Diagnostic::error(format!(
            "i18n key `{local_key}` in `{}` must have `@{local_key}` metadata",
            file.path.display()
        )));
        return;
    };
    validate_description(file, local_key, metadata, diagnostics);
    let actual = metadata_placeholders(metadata);
    if actual != expected {
        diagnostics.push(Diagnostic::error(format!(
            "i18n key `{local_key}` in `{}` has placeholder metadata {:?}, expected {:?}",
            file.path.display(),
            actual,
            expected
        )));
    }
    validate_placeholder_examples(file, local_key, &expected, metadata, diagnostics);
    if message.is_empty() {
        return;
    }
    let message_placeholders = message_placeholders(message);
    if message_placeholders != expected {
        diagnostics.push(Diagnostic::error(format!(
            "i18n key `{local_key}` in `{}` uses placeholders {:?}, expected {:?}",
            file.path.display(),
            message_placeholders,
            expected
        )));
    }
}

/// Validates that message metadata carries a translator-facing description.
fn validate_description(
    file: &CheckedArbFile,
    local_key: &str,
    metadata: &Map<String, Value>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if metadata
        .get("description")
        .and_then(Value::as_str)
        .is_some_and(|description| !description.is_empty())
    {
        return;
    }
    diagnostics.push(Diagnostic::error(format!(
        "i18n metadata `@{local_key}` in `{}` must include a non-empty description",
        file.path.display()
    )));
}

/// Validates that every scanned placeholder has an example.
fn validate_placeholder_examples(
    file: &CheckedArbFile,
    local_key: &str,
    expected: &BTreeSet<String>,
    metadata: &Map<String, Value>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for name in expected {
        let Some(Value::Object(placeholders)) = metadata.get("placeholders") else {
            diagnostics.push(Diagnostic::error(format!(
                "i18n metadata `@{local_key}` in `{}` must include placeholder examples",
                file.path.display()
            )));
            return;
        };
        let Some(Value::Object(placeholder)) = placeholders.get(name) else {
            diagnostics.push(Diagnostic::error(format!(
                "i18n metadata `@{local_key}` in `{}` must define placeholder `{name}`",
                file.path.display()
            )));
            continue;
        };
        if placeholder
            .get("example")
            .and_then(Value::as_str)
            .is_some_and(|example| !example.is_empty())
        {
            continue;
        }
        diagnostics.push(Diagnostic::error(format!(
            "i18n placeholder `{name}` in `@{local_key}` metadata in `{}` must include a non-empty example",
            file.path.display()
        )));
    }
}

/// Returns message metadata for one local key.
fn message_metadata<'a>(
    file: Option<&'a ArbFile>,
    local_key: &str,
) -> Option<&'a Map<String, Value>> {
    let file = file?;
    let metadata_key = format!("@{local_key}");
    let Some(Value::Object(metadata)) = file.map.get(&metadata_key) else {
        return None;
    };
    Some(metadata)
}

/// Returns placeholder names declared in ARB metadata.
fn metadata_placeholders(metadata: &Map<String, Value>) -> BTreeSet<String> {
    let Some(Value::Object(placeholders)) = metadata.get("placeholders") else {
        return BTreeSet::new();
    };
    placeholders.keys().cloned().collect()
}

/// Returns simple `{name}` placeholders from one message string.
fn message_placeholders(message: &str) -> BTreeSet<String> {
    let mut placeholders = BTreeSet::new();
    let mut rest = message;
    while let Some(start) = rest.find('{') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('}') else {
            break;
        };
        let candidate = &after_start[..end];
        if is_arb_identifier(candidate) {
            placeholders.insert(candidate.to_owned());
        }
        rest = &after_start[end + 1..];
    }
    placeholders
}

/// Returns whether one ARB top-level key is a message key.
fn is_message_key(key: &str) -> bool {
    !key.starts_with('@')
}
