use serde_json::{Map, Value};

use crate::i18n_keys::I18nPlannedEntry;

/// Ensures metadata exists for every scanned key.
pub(super) fn ensure_metadata(arb: &mut Map<String, Value>, entry: &I18nPlannedEntry) -> bool {
    let metadata_key = format!("@{}", entry.local_key);
    let Some(metadata) = arb.get_mut(&metadata_key) else {
        arb.insert(metadata_key, metadata_value(entry));
        return true;
    };
    let Value::Object(metadata) = metadata else {
        *metadata = metadata_value(entry);
        return true;
    };
    ensure_metadata_fields(metadata, entry)
}

/// Ensures existing metadata contains generated fields without overwriting custom data.
fn ensure_metadata_fields(metadata: &mut Map<String, Value>, entry: &I18nPlannedEntry) -> bool {
    let mut changed = ensure_description(metadata, entry);
    if entry.args.is_empty() {
        return changed;
    }
    changed |= ensure_placeholder_map(metadata);
    let Some(Value::Object(placeholders)) = metadata.get_mut("placeholders") else {
        return changed;
    };
    for arg in &entry.args {
        changed |= ensure_placeholder_metadata(placeholders, arg);
    }
    changed
}

/// Ensures message metadata has a deterministic description.
fn ensure_description(metadata: &mut Map<String, Value>, entry: &I18nPlannedEntry) -> bool {
    if metadata
        .get("description")
        .and_then(Value::as_str)
        .is_some_and(|description| !description.is_empty())
    {
        return false;
    }
    metadata.insert(
        "description".to_owned(),
        Value::String(description_for(entry)),
    );
    true
}

/// Ensures the placeholder map exists and is an object.
fn ensure_placeholder_map(metadata: &mut Map<String, Value>) -> bool {
    if metadata
        .get("placeholders")
        .is_some_and(serde_json::Value::is_object)
    {
        return false;
    }
    metadata.insert("placeholders".to_owned(), Value::Object(Map::new()));
    true
}

/// Ensures one placeholder has object metadata with an example.
fn ensure_placeholder_metadata(placeholders: &mut Map<String, Value>, name: &str) -> bool {
    let Some(placeholder) = placeholders.get_mut(name) else {
        placeholders.insert(name.to_owned(), placeholder_metadata(name));
        return true;
    };
    let Value::Object(placeholder) = placeholder else {
        *placeholder = placeholder_metadata(name);
        return true;
    };
    if placeholder
        .get("example")
        .and_then(Value::as_str)
        .is_some_and(|example| !example.is_empty())
    {
        return false;
    }
    placeholder.insert("example".to_owned(), Value::String(example_for(name)));
    true
}

/// Builds ARB metadata for one message.
fn metadata_value(entry: &I18nPlannedEntry) -> Value {
    let mut metadata = Map::new();
    metadata.insert(
        "description".to_owned(),
        Value::String(description_for(entry)),
    );
    if entry.args.is_empty() {
        return Value::Object(metadata);
    }
    let mut placeholders = Map::new();
    for arg in &entry.args {
        placeholders.insert(arg.clone(), placeholder_metadata(arg));
    }
    metadata.insert("placeholders".to_owned(), Value::Object(placeholders));
    Value::Object(metadata)
}

/// Builds a deterministic generated message description.
fn description_for(entry: &I18nPlannedEntry) -> String {
    format!("Translation for `{}`.", entry.key)
}

/// Builds deterministic placeholder metadata without guessing semantic types.
fn placeholder_metadata(name: &str) -> Value {
    let mut metadata = Map::new();
    metadata.insert("example".to_owned(), Value::String(example_for(name)));
    Value::Object(metadata)
}

/// Returns a stable placeholder example from a placeholder name.
fn example_for(name: &str) -> String {
    let normalized = name.to_ascii_lowercase();
    if normalized.contains("count") || normalized.contains("quantity") {
        return "1".to_owned();
    }
    if normalized.contains("price") || normalized.contains("amount") || normalized.contains("total")
    {
        return "9.99".to_owned();
    }
    if normalized.contains("date") || normalized.contains("time") {
        return "2026-01-01".to_owned();
    }
    if normalized.contains("email") {
        return "user@example.com".to_owned();
    }
    if normalized.contains("currency") {
        return "USD".to_owned();
    }
    if normalized.contains("name") || normalized.contains("user") {
        return "Alice".to_owned();
    }
    name.to_owned()
}
