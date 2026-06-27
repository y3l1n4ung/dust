use dust_dart_emit::{dart_string_literal, render_template};
use dust_ir::{EnumIr, EnumVariantIr};
use serde::Serialize;

/// Template context for generated enum JSON helpers.
#[derive(Serialize)]
struct EnumTemplateContext<'a> {
    /// Source enum name.
    enum_name: &'a str,
    /// Rendered switch cases.
    cases: String,
}

/// Renders the top-level helper that deserializes an enum value.
pub(crate) fn emit_enum_from_json_helper(e: &EnumIr) -> String {
    let mut cases = Vec::new();
    for variant in &e.variants {
        if let Some(key) = variant_wire_name(e, variant) {
            cases.push(format!(
                "    {} => {}.{},",
                dart_string_literal(&key),
                e.name,
                variant.name
            ));
        }
    }

    render_template(
        "enum_from_json",
        include_str!("templates/enum_from_json.jinja"),
        EnumTemplateContext {
            enum_name: &e.name,
            cases: cases.join("\n"),
        },
    )
}

/// Renders the top-level helper that serializes an enum value.
pub(crate) fn emit_enum_to_json_helper(e: &EnumIr) -> String {
    let mut cases = Vec::new();
    for variant in &e.variants {
        if let Some(key) = variant_wire_name(e, variant) {
            cases.push(format!(
                "    {}.{} => {},",
                e.name,
                variant.name,
                dart_string_literal(&key)
            ));
        }
    }
    if e.variants
        .iter()
        .any(|variant| variant.serde.as_ref().is_some_and(|serde| serde.skip))
    {
        cases.push(format!(
            "    _ => throw ArgumentError.value(instance, 'instance', 'skipped value for {}'),",
            e.name
        ));
    }

    render_template(
        "enum_to_json",
        include_str!("templates/enum_to_json.jinja"),
        EnumTemplateContext {
            enum_name: &e.name,
            cases: cases.join("\n"),
        },
    )
}

/// Resolves the wire value for an enum variant.
pub(crate) fn variant_wire_name(e: &EnumIr, variant: &EnumVariantIr) -> Option<String> {
    let serde = variant.serde.as_ref();
    if serde.is_some_and(|serde| serde.skip) {
        return None;
    }
    if let Some(rename) = serde.and_then(|serde| serde.rename.as_ref()).cloned() {
        return Some(rename);
    }

    Some(match e.serde.as_ref().and_then(|s| s.rename_all) {
        Some(rule) => crate::writer::apply_rename_rule(&variant.name, rule),
        None => variant.name.clone(),
    })
}
