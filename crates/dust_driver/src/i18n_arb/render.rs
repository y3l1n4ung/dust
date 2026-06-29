use std::collections::BTreeSet;

use dust_diagnostics::Diagnostic;
use serde_json::{Map, Value};

/// Renders an ARB object in deterministic order.
pub(super) fn render_arb(arb: &Map<String, Value>) -> Result<String, Diagnostic> {
    let keys = ordered_keys(arb);
    let mut output = String::from("{\n");
    for (index, key) in keys.iter().enumerate() {
        let Some(value) = arb.get(key) else {
            continue;
        };
        output.push_str("  ");
        output.push_str(&json_string(key)?);
        output.push_str(": ");
        output.push_str(&indent_json_value(value)?);
        if index + 1 != keys.len() {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("}\n");
    Ok(output)
}

/// Returns top-level ARB keys in deterministic ARB-friendly order.
fn ordered_keys(arb: &Map<String, Value>) -> Vec<String> {
    let mut emitted = BTreeSet::<String>::new();
    let mut ordered = Vec::<String>::new();
    push_if_present("@@locale", arb, &mut emitted, &mut ordered);

    for key in sorted_keys(arb)
        .into_iter()
        .filter(|key| key.starts_with("@@") && key.as_str() != "@@locale")
    {
        push_key(key, &mut emitted, &mut ordered);
    }

    for key in sorted_keys(arb)
        .into_iter()
        .filter(|key| !key.starts_with('@'))
    {
        let metadata_key = format!("@{key}");
        push_key(key, &mut emitted, &mut ordered);
        push_if_present(&metadata_key, arb, &mut emitted, &mut ordered);
    }

    for key in sorted_keys(arb)
        .into_iter()
        .filter(|key| key.starts_with('@') && !key.starts_with("@@"))
    {
        push_key(key, &mut emitted, &mut ordered);
    }

    ordered
}

/// Returns sorted top-level ARB keys.
fn sorted_keys(arb: &Map<String, Value>) -> Vec<String> {
    let mut keys = arb.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}

/// Pushes one key when it exists in an ARB object.
fn push_if_present(
    key: &str,
    arb: &Map<String, Value>,
    emitted: &mut BTreeSet<String>,
    ordered: &mut Vec<String>,
) {
    if arb.contains_key(key) {
        push_key(key.to_owned(), emitted, ordered);
    }
}

/// Pushes one key unless it was already emitted.
fn push_key(key: String, emitted: &mut BTreeSet<String>, ordered: &mut Vec<String>) {
    if emitted.insert(key.clone()) {
        ordered.push(key);
    }
}

/// Renders one JSON string.
fn json_string(value: &str) -> Result<String, Diagnostic> {
    serde_json::to_string(value)
        .map_err(|error| Diagnostic::error(format!("failed to render ARB key: {error}")))
}

/// Renders and indents one JSON value.
fn indent_json_value(value: &Value) -> Result<String, Diagnostic> {
    let rendered = serde_json::to_string_pretty(value)
        .map_err(|error| Diagnostic::error(format!("failed to render ARB value: {error}")))?;
    Ok(rendered.replace('\n', "\n  "))
}
