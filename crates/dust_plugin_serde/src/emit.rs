use std::collections::HashSet;

use dust_ir::{ClassIr, EnumIr, LibraryIr};
use dust_plugin_api::PluginContribution;
use heck::AsPascalCase;

use crate::writer::{
    all_allowed_keys, decode_field_expr, encode_field_expr, find_deserialize_constructor, json_key,
    render_constructor_call,
};

/// Orchestrates the emission of all SerDe-related code for a library.
///
/// This function identifies which models (classes and enums) have requested
/// serialization or deserialization and generates the corresponding Dart code.
pub(crate) fn emit_library(library: &LibraryIr) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let serializable_classes = library
        .classes
        .iter()
        .filter(|class| wants_serialize(class))
        .map(|class| class.name.clone())
        .collect::<HashSet<_>>();
    let serializable_enums = library
        .enums
        .iter()
        .filter(|e| wants_serialize_enum(e))
        .map(|e| e.name.clone())
        .collect::<HashSet<_>>();

    let deserializable_classes = library
        .classes
        .iter()
        .filter(|class| wants_deserialize(class))
        .map(|class| class.name.clone())
        .collect::<HashSet<_>>();
    let deserializable_enums = library
        .enums
        .iter()
        .filter(|e| wants_deserialize_enum(e))
        .map(|e| e.name.clone())
        .collect::<HashSet<_>>();

    // If any model needs deserialization, include the standard shared JSON helpers.
    if !deserializable_classes.is_empty() || !deserializable_enums.is_empty() {
        contribution
            .shared_helpers
            .push(render_deserialize_helpers().to_owned());
    }

    // Generate class-specific code.
    for class in &library.classes {
        if wants_serialize(class) {
            contribution.push_mixin_member(&class.name, emit_to_json_mixin(class));
            contribution.top_level_functions.push(emit_to_json_helper(
                class,
                &serializable_classes,
                &serializable_enums,
            ));
        }
        if wants_deserialize(class) {
            if let Some(helper) = emit_from_json_helper(
                class,
                &deserializable_classes,
                &deserializable_enums,
            ) {
                contribution.top_level_functions.push(helper);
            }
        }
    }

    // Generate enum-specific code.
    for e in &library.enums {
        if wants_serialize_enum(e) {
            contribution
                .top_level_functions
                .push(emit_enum_to_json_helper(e));
        }
        if wants_deserialize_enum(e) {
            contribution
                .top_level_functions
                .push(emit_enum_from_json_helper(e));
        }
    }
    contribution
}

/// Generates the `_$EnumNameFromJson` helper for Dart enums.
///
/// Enums are deserialized from their string representations, mapping back
/// to the appropriate variant.
fn emit_enum_from_json_helper(e: &EnumIr) -> String {
    let mut cases = Vec::new();
    for variant in &e.variants {
        let key = match e.serde.as_ref().and_then(|s| s.rename_all) {
            Some(rule) => crate::writer::apply_rename_rule(&variant.name, rule),
            None => variant.name.clone(),
        };
        cases.push(format!("    '{}' => {}.{},", key, e.name, variant.name));
    }

    format!(
        "{} _${}FromJson(Object? json) {{\n  return switch (json) {{\n{}\n    _ => throw ArgumentError.value(json, 'json', 'unknown value for {}'),\n  }};\n}}",
        e.name, e.name, cases.join("\n"), e.name
    )
}

/// Generates the `_$EnumNameToJson` helper for Dart enums.
///
/// Enums are serialized to their string representations based on the variant name
/// and any applied rename rules.
fn emit_enum_to_json_helper(e: &EnumIr) -> String {
    let mut cases = Vec::new();
    for variant in &e.variants {
        let key = match e.serde.as_ref().and_then(|s| s.rename_all) {
            Some(rule) => crate::writer::apply_rename_rule(&variant.name, rule),
            None => variant.name.clone(),
        };
        cases.push(format!("    {}.{} => '{}',", e.name, variant.name, key));
    }

    format!(
        "Object? _${}ToJson({} instance) {{\n  return switch (instance) {{\n{}\n  }};\n}}",
        e.name,
        e.name,
        cases.join("\n")
    )
}

/// Generates the `toJson()` mixin member for a class.
fn emit_to_json_mixin(class: &ClassIr) -> String {
    format!(
        "Map<String, Object?> toJson() => _${}ToJson(_dustSelf);",
        class.name
    )
}

/// Generates the private top-level `_$ClassNameToJson` helper for a class.
///
/// This helper performs the actual mapping of fields to a JSON-compatible Map.
fn emit_to_json_helper(
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

/// Generates the private top-level `_$ClassNameFromJson` helper for a class.
///
/// This helper handles field extraction from a JSON Map, including alias resolution,
/// default value application, and nested model deserialization.
fn emit_from_json_helper(
    class: &ClassIr,
    deserializable_classes: &HashSet<String>,
    deserializable_enums: &HashSet<String>,
) -> Option<String> {
    let constructor = find_deserialize_constructor(class)?;
    let mut lines = Vec::new();

    // Key validation if `disallowUnrecognizedKeys` is enabled.
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
        
        // Skip field if explicitly ignored for deserialization.
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

        // Extract the raw value from the JSON Map.
        let decoded = if aliases.is_empty() {
            decode_field_expr(
                &format!("json['{primary_key}']"),
                &format!("'{primary_key}'"),
                field,
                deserializable_classes,
                deserializable_enums,
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
        };

        // Apply default value if key is missing and default is provided.
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

    // Call the chosen constructor with the extracted values.
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

fn wants_serialize_enum(e: &EnumIr) -> bool {
    e.traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Serialize")
}

fn wants_deserialize_enum(e: &EnumIr) -> bool {
    e.traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Deserialize")
}

/// Formats a multi-line expression with an indentation-aware prefix.
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

/// Standard shared helper functions generated into every SerDe-enabled library.
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
