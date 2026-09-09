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

/// Emitting the decode side of a generated codec, field by field.
mod decode;
use self::decode::*;

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

/// Renders the generated `serialize` and `toJson` mixin members for a class.
pub(crate) fn emit_serialize_mixin_members(helper_class_name: &str) -> Vec<String> {
    let inline = format!(
        "Map<String, Object?> serialize() => _${helper_class_name}Serialize(this as {helper_class_name});"
    );
    let serialize = if inline.len() <= MIXIN_MEMBER_WIDTH {
        inline
    } else {
        format!(
            "Map<String, Object?> serialize() =>\n    _${helper_class_name}Serialize(this as {helper_class_name});"
        )
    };

    vec![
        serialize,
        "Map<String, Object?> toJson() => serialize();".to_owned(),
    ]
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

/// Renders the generated serializer support class for a class.
pub(crate) fn emit_serializer_support_type(class_name: &str) -> String {
    format!(
        r#"final class ${class_name}Serializer implements Serializer<{class_name}, Map<String, Object?>> {{
  const ${class_name}Serializer();

  @override
  Map<String, Object?> serialize({class_name} value) => _${class_name}Serialize(value);
}}"#
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

/// Renders the generated deserializer support class for a class.
pub(crate) fn emit_deserializer_support_type(class_name: &str) -> String {
    format!(
        r#"final class ${class_name}Deserializer implements Deserializer<{class_name}, Map<String, Object?>> {{
  const ${class_name}Deserializer();

  @override
  {class_name} deserialize(Map<String, Object?> json) => _${class_name}Deserialize(json);
}}"#
    )
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
