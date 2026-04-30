use std::collections::HashSet;

use dust_ir::{ClassIr, LibraryIr};
use dust_plugin_api::PluginContribution;

use crate::writer::{
    all_allowed_keys, decode_field_expr, encode_field_expr, find_deserialize_constructor, json_key,
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

    if library.classes.iter().any(wants_deserialize) {
        contribution
            .shared_helpers
            .push(render_deserialize_helpers().to_owned());
    }

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
        let value = encode_field_expr(
            &format!("instance.{}", field.name),
            field,
            serializable_models,
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
            lines.push(format_prefixed_expr(
                2,
                &format!("final {field_var} = "),
                &default,
                ";",
            ));
            values.push((field.name.as_str(), field_var.clone()));
            continue;
        }

        let primary_key = json_key(class, &field.name, serde);
        let aliases = serde.map(|serde| serde.aliases.as_slice()).unwrap_or(&[]);
        let has_expr = if aliases.is_empty() {
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
        };

        let decoded = if aliases.is_empty() {
            decode_field_expr(
                &format!("json['{primary_key}']"),
                &format!("'{primary_key}'"),
                field,
                deserializable_models,
            )
        } else {
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
            let raw_name = format!("raw{}", capitalize(&field.name));
            let raw_key_name = format!("raw{}Key", capitalize(&field.name));
            let key_expr = format!("{} : '{primary_key}'", key_parts.join(" : "));
            let raw_expr = format!("{} : null", raw_parts.join(" : "));
            lines.push(format_prefixed_expr(
                2,
                &format!("final {raw_key_name} = "),
                &key_expr,
                ";",
            ));
            lines.push(format_prefixed_expr(
                2,
                &format!("final {raw_name} = "),
                &raw_expr,
                ";",
            ));
            decode_field_expr(&raw_name, &raw_key_name, field, deserializable_models)
        };
        let value_expr = if let Some(default_value) =
            serde.and_then(|serde| serde.default_value_source.as_deref())
        {
            format!("{has_expr} ? {decoded} : {default_value}")
        } else {
            decoded
        };
        lines.push(format_prefixed_expr(
            2,
            &format!("final {field_var} = "),
            &value_expr,
            ";",
        ));
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
        "// factory {0}.fromJson(Map<String, Object?> json) => _${0}FromJson(json);\n\
         {0} _${0}FromJson(Map<String, Object?> json) {{\n{1}\n}}",
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

fn format_prefixed_expr(indent: usize, prefix: &str, expr: &str, suffix: &str) -> String {
    let pad = " ".repeat(indent);
    let continuation = " ".repeat(indent + prefix.len());
    let mut lines = expr.lines();
    let Some(first) = lines.next() else {
        return format!("{pad}{prefix}{suffix}");
    };

    let rest = lines.collect::<Vec<_>>();
    if rest.is_empty() {
        return format!("{pad}{prefix}{first}{suffix}");
    }

    let mut rendered = Vec::with_capacity(rest.len() + 1);
    rendered.push(format!("{pad}{prefix}{first}"));
    for (index, line) in rest.iter().enumerate() {
        let tail = if index + 1 == rest.len() { suffix } else { "" };
        rendered.push(format!("{continuation}{line}{tail}"));
    }
    rendered.join("\n")
}

fn render_deserialize_helpers() -> &'static str {
    r#"Never _dustJsonTypeError(Object? value, String key, String expected) => throw ArgumentError.value(value, key, 'expected $expected');
T _dustJsonAs<T>(Object? value, String key, String expected) => value is T ? value : _dustJsonTypeError(value, key, expected);
T _dustJsonParseString<T>(Object? value, String key, String expected, T? Function(String value) parse) => parse(_dustJsonAs<String>(value, key, 'String')) ?? _dustJsonTypeError(value, key, expected);
List<Object?> _dustJsonAsList(Object? value, String key) => _dustJsonAs<List>(value, key, 'List<Object?>').cast<Object?>();

Map<String, Object?> _dustJsonAsMap(Object? value, String key) {
  final map = _dustJsonAs<Map>(value, key, 'Map<String, Object?>');
  try {
    return Map<String, Object?>.from(map);
  } on TypeError {
    _dustJsonTypeError(value, key, 'Map<String, Object?>');
  }
}
DateTime _dustJsonAsDateTime(Object? value, String key) => _dustJsonParseString(value, key, 'ISO-8601 DateTime string', DateTime.tryParse);
Uri _dustJsonAsUri(Object? value, String key) => _dustJsonParseString(value, key, 'Uri string', Uri.tryParse);
BigInt _dustJsonAsBigInt(Object? value, String key) => _dustJsonParseString(value, key, 'BigInt string', BigInt.tryParse);
T _dustJsonDecodeWithCodec<T>(dynamic codec, Object? value, String key) {
  if (value == null) {
    throw ArgumentError.value(value, key, 'expected value for SerDeCodec');
  }
  try {
    return codec.deserialize(value as dynamic) as T;
  } catch (error) {
    throw ArgumentError.value(value, key, 'failed SerDeCodec decode: $error');
  }
}"#
}
