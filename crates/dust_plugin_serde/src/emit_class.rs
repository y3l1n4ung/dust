use std::collections::HashSet;

use dust_ir::ClassIr;
use heck::AsPascalCase;

use crate::{
    emit_support::format_prefixed_expr,
    writer::{
        all_allowed_keys, decode_field_expr, encode_field_expr, find_deserialize_constructor,
        json_key, render_constructor_call,
    },
};

pub(crate) fn emit_to_json_mixin(class: &ClassIr) -> String {
    format!(
        "Map<String, Object?> toJson() => _${}ToJson(_dustSelf);",
        class.name
    )
}

pub(crate) fn emit_to_json_helper(
    class: &ClassIr,
    serializable_classes: &HashSet<String>,
    serializable_enums: &HashSet<String>,
) -> String {
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
        let value = encode_field_expr(
            &format!("instance.{}", field.name),
            field,
            serializable_classes,
            serializable_enums,
        );
        lines.push(format_prefixed_expr(4, &format!("'{key}': "), &value, ","));
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

pub(crate) fn emit_from_json_helper(
    class: &ClassIr,
    deserializable_classes: &HashSet<String>,
    deserializable_enums: &HashSet<String>,
) -> Option<String> {
    let constructor = find_deserialize_constructor(class)?;
    let mut lines = Vec::new();

    if class
        .serde
        .as_ref()
        .is_some_and(|serde| serde.disallow_unrecognized_keys)
    {
        emit_allowed_key_validation(class, &mut lines);
    }

    let mut values = Vec::new();
    for field in &class.fields {
        let field_var = format!("{}Value", field.name);
        let value_expr = emit_field_decode(
            class,
            field,
            deserializable_classes,
            deserializable_enums,
            &mut lines,
        );
        lines.push(format_prefixed_expr(
            2,
            &format!("final {field_var} = "),
            &value_expr,
            ";",
        ));
        values.push((field.name.as_str(), field_var));
    }

    let call = render_constructor_call(class, constructor, &values)?;
    append_constructor_return(&mut lines, &call);

    Some(format!(
        "// factory {0}.fromJson(Map<String, Object?> json) => _${0}FromJson(json);\n\
         {0} _${0}FromJson(Map<String, Object?> json) {{\n{1}\n}}",
        class.name,
        lines.join("\n")
    ))
}

fn emit_allowed_key_validation(class: &ClassIr, lines: &mut Vec<String>) {
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

fn emit_field_decode(
    class: &ClassIr,
    field: &dust_ir::FieldIr,
    deserializable_classes: &HashSet<String>,
    deserializable_enums: &HashSet<String>,
    lines: &mut Vec<String>,
) -> String {
    let serde = field.serde.as_ref();
    if serde.is_some_and(|serde| serde.skip_deserializing) {
        return serde
            .and_then(|serde| serde.default_value_source.clone())
            .unwrap_or_else(|| "null".to_owned());
    }

    let primary_key = json_key(class, &field.name, serde);
    let aliases = serde.map(|serde| serde.aliases.as_slice()).unwrap_or(&[]);
    let has_expr = build_has_expr(&primary_key, aliases);
    let decoded = if aliases.is_empty() {
        decode_field_expr(
            &format!("json['{primary_key}']"),
            &format!("'{primary_key}'"),
            field,
            deserializable_classes,
            deserializable_enums,
        )
    } else {
        emit_alias_decode(
            field,
            &primary_key,
            aliases,
            deserializable_classes,
            deserializable_enums,
            lines,
        )
    };

    serde
        .and_then(|serde| serde.default_value_source.as_deref())
        .map_or(decoded.clone(), |default_value| {
            format!("{has_expr} ? {decoded} : {default_value}")
        })
}

fn build_has_expr(primary_key: &str, aliases: &[String]) -> String {
    if aliases.is_empty() {
        format!("json.containsKey('{primary_key}')")
    } else {
        std::iter::once(format!("json.containsKey('{primary_key}')"))
            .chain(
                aliases
                    .iter()
                    .map(|alias| format!("json.containsKey('{alias}')")),
            )
            .collect::<Vec<_>>()
            .join(" || ")
    }
}

fn emit_alias_decode(
    field: &dust_ir::FieldIr,
    primary_key: &str,
    aliases: &[String],
    deserializable_classes: &HashSet<String>,
    deserializable_enums: &HashSet<String>,
    lines: &mut Vec<String>,
) -> String {
    let key_parts = std::iter::once(format!(
        "json.containsKey('{primary_key}') ? '{primary_key}'"
    ))
    .chain(
        aliases
            .iter()
            .map(|alias| format!("json.containsKey('{alias}') ? '{alias}'")),
    )
    .collect::<Vec<_>>();
    let raw_parts = std::iter::once(format!(
        "json.containsKey('{primary_key}') ? json['{primary_key}']"
    ))
    .chain(
        aliases
            .iter()
            .map(|alias| format!("json.containsKey('{alias}') ? json['{alias}']")),
    )
    .collect::<Vec<_>>();
    let raw_name = format!("raw{}", AsPascalCase(&field.name));
    let raw_key_name = format!("raw{}Key", AsPascalCase(&field.name));
    let key_expr = format!("{} : '{primary_key}'", key_parts.join(" : "));
    let raw_expr = format!("{} : null", raw_parts.join(" : "));
    let decoded = decode_field_expr(
        &raw_name,
        &raw_key_name,
        field,
        deserializable_classes,
        deserializable_enums,
    );
    if decoded.contains(&raw_key_name) {
        lines.push(format_prefixed_expr(
            2,
            &format!("final {raw_key_name} = "),
            &key_expr,
            ";",
        ));
    }
    lines.push(format_prefixed_expr(
        2,
        &format!("final {raw_name} = "),
        &raw_expr,
        ";",
    ));
    decoded
}

fn append_constructor_return(lines: &mut Vec<String>, call: &str) {
    lines.push(String::new());
    lines.push(format!(
        "  return {};",
        call.lines().next().unwrap_or_default()
    ));
    if call.lines().count() <= 1 {
        return;
    }

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
