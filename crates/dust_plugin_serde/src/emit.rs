use std::collections::HashSet;

use dust_ir::{ClassIr, LibraryIr};
use dust_plugin_api::PluginContribution;

use crate::writer::{
    all_allowed_keys, decode_expr, encode_expr, find_deserialize_constructor, json_key,
    render_constructor_call,
};

pub(crate) fn emit_library(library: &LibraryIr) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let serializable_models = library
        .classes
        .iter()
        .filter(|class| wants_serialize(class))
        .map(|class| class.name.clone())
        .collect::<HashSet<_>>();
    let deserializable_models = library
        .classes
        .iter()
        .filter(|class| wants_deserialize(class))
        .map(|class| class.name.clone())
        .collect::<HashSet<_>>();

    for class in &library.classes {
        if wants_serialize(class) {
            contribution.push_mixin_member(&class.name, emit_to_json_mixin(class));
            contribution
                .top_level_functions
                .push(emit_to_json_helper(class, &serializable_models));
        }
        if wants_deserialize(class) {
            if let Some(helper) = emit_from_json_helper(class, &deserializable_models) {
                contribution.top_level_functions.push(helper);
            }
        }
    }

    contribution
}

fn emit_to_json_mixin(class: &ClassIr) -> String {
    format!(
        "Map<String, Object?> toJson() => _${}ToJson(_dustSelf);",
        class.name
    )
}

fn emit_to_json_helper(class: &ClassIr, serializable_models: &HashSet<String>) -> String {
    let mut lines = Vec::new();
    for field in &class.fields {
        if field
            .serde
            .as_ref()
            .is_some_and(|serde| serde.skip_serializing)
        {
            continue;
        }

        let key = json_key(class, &field.name, field.serde.as_ref());
        let value = encode_expr(
            &format!("instance.{}", field.name),
            &field.ty,
            serializable_models,
        );
        lines.push(format!("    '{key}': {value},"));
    }

    let body = if lines.is_empty() {
        "  return <String, Object?>{};".to_owned()
    } else {
        format!("  return <String, Object?>{{\n{}\n  }};", lines.join("\n"))
    };

    format!(
        "Map<String, Object?> _${}ToJson({} instance) {{\n{}\n}}",
        class.name, class.name, body
    )
}

fn emit_from_json_helper(
    class: &ClassIr,
    deserializable_models: &HashSet<String>,
) -> Option<String> {
    let constructor = find_deserialize_constructor(class)?;
    let mut lines = Vec::new();

    if class
        .serde
        .as_ref()
        .is_some_and(|serde| serde.disallow_unrecognized_keys)
    {
        let allowed_keys = all_allowed_keys(class)
            .into_iter()
            .map(|key| format!("'{key}'"))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("  const allowedKeys = <String>{{{allowed_keys}}};"));
        lines.push("  for (final key in json.keys) {".to_owned());
        lines.push("    if (!allowedKeys.contains(key)) {".to_owned());
        lines.push(format!(
            "      throw ArgumentError.value(key, 'json', 'unknown key for {}');",
            class.name
        ));
        lines.push("    }".to_owned());
        lines.push("  }".to_owned());
        lines.push(String::new());
    }

    let mut values = Vec::new();
    for field in &class.fields {
        let serde = field.serde.as_ref();
        let field_var = format!("{}Value", field.name);
        if serde.is_some_and(|serde| serde.skip_deserializing) {
            let default = serde
                .and_then(|serde| serde.default_value_source.clone())
                .unwrap_or_else(|| "null".to_owned());
            lines.push(format!("  final {field_var} = {default};"));
            values.push((field.name.as_str(), field_var.clone()));
            continue;
        }

        let primary_key = json_key(class, &field.name, serde);
        let mut has_parts = vec![format!("json.containsKey('{primary_key}')")];
        let mut raw_parts = vec![format!(
            "json.containsKey('{primary_key}') ? json['{primary_key}']"
        )];
        if let Some(serde) = serde {
            for alias in &serde.aliases {
                has_parts.push(format!("json.containsKey('{alias}')"));
                raw_parts.push(format!("json.containsKey('{alias}') ? json['{alias}']"));
            }
        }
        let has_expr = has_parts.join(" || ");
        let raw_expr = format!("{} : null", raw_parts.join(" : "));
        let raw_name = format!("raw{}", capitalize(&field.name));
        lines.push(format!("  final {raw_name} = {raw_expr};"));

        let decoded = decode_expr(&raw_name, &field.ty, deserializable_models);
        let value_expr = if let Some(default_value) =
            serde.and_then(|serde| serde.default_value_source.as_deref())
        {
            format!("{has_expr} ? {decoded} : {default_value}")
        } else {
            decoded
        };
        lines.push(format!("  final {field_var} = {value_expr};"));
        values.push((field.name.as_str(), field_var));
    }

    let call = render_constructor_call(class, constructor, &values)?;
    lines.push(String::new());
    lines.push(format!(
        "  return {};",
        call.lines().next().unwrap_or_default()
    ));
    if call.lines().count() > 1 {
        lines.pop();
        let mut call_lines = call.lines();
        if let Some(first) = call_lines.next() {
            lines.push(format!("  return {first}"));
        }
        for line in call_lines {
            if line == ")" {
                lines.push(format!("  {line};"));
            } else {
                lines.push(format!("  {line}"));
            }
        }
    }

    Some(format!(
        "{} _${}FromJson(Map<String, Object?> json) {{\n{}\n}}",
        class.name,
        class.name,
        lines.join("\n")
    ))
}

fn wants_serialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Serialize")
}

fn wants_deserialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Deserialize")
}

fn capitalize(source: &str) -> String {
    let mut chars = source.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
}
