use dust_dart_emit::render_template;
use dust_ir::ClassIr;
use heck::AsPascalCase;
use serde::Serialize;

use crate::{
    emit_support::format_prefixed_expr,
    writer::{
        all_allowed_keys, decode_field_expr, encode_field_expr, find_deserialize_constructor,
        json_key, render_constructor_call,
    },
};

/// Dart's standard formatter line width.
const DART_LINE_WIDTH: usize = 80;

/// Width available before the enclosing mixin adds two spaces of indentation.
const MIXIN_MEMBER_WIDTH: usize = DART_LINE_WIDTH - 2;

/// Template context for generated class JSON helpers.
#[derive(Serialize)]
struct ClassTemplateContext<'a> {
    /// Source Dart class name.
    class_name: &'a str,
    /// Rendered helper body.
    body: String,
}

/// Renders the generated `toJson` mixin member for a class.
pub(crate) fn emit_to_json_mixin(class: &ClassIr) -> String {
    let inline = format!(
        "Map<String, Object?> toJson() => _${}ToJson(this as {});",
        class.name, class.name
    );
    if inline.len() <= MIXIN_MEMBER_WIDTH {
        return inline;
    }

    format!(
        "Map<String, Object?> toJson() =>\n    _${}ToJson(this as {});",
        class.name, class.name
    )
}

/// Renders the top-level helper that serializes a class instance.
pub(crate) fn emit_to_json_helper(
    class: &ClassIr,
    serializable_classes: &[&str],
    serializable_enums: &[&str],
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

    render_template(
        "to_json_helper",
        include_str!("templates/to_json_helper.jinja"),
        ClassTemplateContext {
            class_name: &class.name,
            body,
        },
    )
}

/// Renders the top-level helper that deserializes a class instance.
pub(crate) fn emit_from_json_helper(
    class: &ClassIr,
    deserializable_classes: &[&str],
    deserializable_enums: &[&str],
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

    Some(render_template(
        "from_json_helper",
        include_str!("templates/from_json_helper.jinja"),
        ClassTemplateContext {
            class_name: &class.name,
            body: lines.join("\n"),
        },
    ))
}

/// Emits runtime validation for allowed JSON keys.
fn emit_allowed_key_validation(class: &ClassIr, lines: &mut Vec<String>) {
    let allowed_keys = all_allowed_keys(class);
    if allowed_keys.len() <= 4 {
        let allowed_keys = allowed_keys
            .into_iter()
            .map(|key| format!("'{key}'"))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("  const allowedKeys = <String>{{{allowed_keys}}};"));
    } else {
        lines.push("  const allowedKeys = <String>{".to_owned());
        for key in allowed_keys {
            lines.push(format!("    '{key}',"));
        }
        lines.push("  };".to_owned());
    }
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

/// Emits the expression that decodes one class field.
fn emit_field_decode(
    class: &ClassIr,
    field: &dust_ir::FieldIr,
    deserializable_classes: &[&str],
    deserializable_enums: &[&str],
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
            let inline = format!("{has_expr} ? {decoded} : {default_value}");
            if inline.len() <= 80 {
                inline
            } else {
                format!("{has_expr}\n    ? {decoded}\n    : {default_value}")
            }
        })
}

/// Builds the expression that checks primary and alias keys.
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

/// Emits alias-aware decode setup and returns the final decode expression.
fn emit_alias_decode(
    field: &dust_ir::FieldIr,
    primary_key: &str,
    aliases: &[String],
    deserializable_classes: &[&str],
    deserializable_enums: &[&str],
    lines: &mut Vec<String>,
) -> String {
    let raw_name = format!("raw{}", AsPascalCase(&field.name));
    let raw_key_name = format!("raw{}Key", AsPascalCase(&field.name));
    let decoded = decode_field_expr(
        &raw_name,
        &raw_key_name,
        field,
        deserializable_classes,
        deserializable_enums,
    );
    let uses_raw_key = decoded.contains(&raw_key_name);

    if uses_raw_key {
        lines.push(format!("  var {raw_key_name} = '{primary_key}';"));
    }
    lines.push(format!("  Object? {raw_name};"));
    lines.push(format!("  if (json.containsKey('{primary_key}')) {{"));
    lines.push(format!("    {raw_name} = json['{primary_key}'];"));
    for alias in aliases {
        lines.push(format!("  }} else if (json.containsKey('{alias}')) {{"));
        if uses_raw_key {
            lines.push(format!("    {raw_key_name} = '{alias}';"));
        }
        lines.push(format!("    {raw_name} = json['{alias}'];"));
    }
    lines.push("  }".to_owned());

    decoded
}

/// Appends a constructor return statement, preserving multiline formatting.
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
