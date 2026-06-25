use dust_dart_emit::dart_string_literal;
use dust_ir::{ClassIr, SerdeClassConfigIr, SerdeVariantConfigIr};

/// Returns whether a class should use tagged sealed dispatch helpers.
pub(crate) fn is_tagged_sealed_class(class: &ClassIr) -> bool {
    tagged_serde(class).is_some()
}

/// Renders the top-level helper that serializes a tagged sealed base instance.
pub(crate) fn emit_sealed_to_json_helper(
    class: &ClassIr,
    serializable_classes: &[&str],
) -> Option<String> {
    let serde = tagged_serde(class)?;
    let tag_key = serde.tag.as_deref()?;

    let mut lines = vec![
        format!(
            "Map<String, Object?> _${}ToJson({} instance) {{",
            class.name, class.name
        ),
        "  return switch (instance) {".to_owned(),
    ];
    for variant in &serde.variants {
        append_to_json_case(
            &mut lines,
            variant,
            tag_key,
            serde.content.as_deref(),
            serializable_classes,
        );
    }
    lines.push("  };".to_owned());
    lines.push("}".to_owned());
    Some(lines.join("\n"))
}

/// Renders the top-level helper that deserializes a tagged sealed base instance.
pub(crate) fn emit_sealed_from_json_helper(
    class: &ClassIr,
    deserializable_classes: &[&str],
) -> Option<String> {
    let serde = tagged_serde(class)?;
    let tag_key = serde.tag.as_deref()?;
    let tag_key_lit = dart_string_literal(tag_key);

    let mut lines = vec![
        format!(
            "// factory {}.fromJson(Map<String, Object?> json) => _${}FromJson(json);",
            class.name, class.name
        ),
        format!(
            "{} _${}FromJson(Map<String, Object?> json) {{",
            class.name, class.name
        ),
        format!(
            "  final tagValue = JsonHelper.as<String>(json[{tag_key_lit}], {tag_key_lit}, 'String');"
        ),
    ];

    let variant_json = if let Some(content_key) = serde.content.as_deref() {
        let content_key_lit = dart_string_literal(content_key);
        lines.push(format!(
            "  final contentValue = JsonHelper.asMap(json[{content_key_lit}], {content_key_lit});"
        ));
        "contentValue".to_owned()
    } else {
        lines.push(format!(
            "  final variantJson = Map<String, Object?>.from(json)..remove({tag_key_lit});"
        ));
        "variantJson".to_owned()
    };

    lines.push(String::new());
    lines.push("  return switch (tagValue) {".to_owned());
    for variant in &serde.variants {
        lines.push(format!(
            "    {} => {},",
            dart_string_literal(&variant.tag),
            variant_from_json_expr(variant, &variant_json, deserializable_classes)
        ));
    }
    lines.push("    _ => throw ArgumentError('Unknown SerDe variant tag: $tagValue'),".to_owned());
    lines.push("  };".to_owned());
    lines.push("}".to_owned());
    Some(lines.join("\n"))
}

/// Appends one sealed serialization switch case.
fn append_to_json_case(
    lines: &mut Vec<String>,
    variant: &SerdeVariantConfigIr,
    tag_key: &str,
    content_key: Option<&str>,
    serializable_classes: &[&str],
) {
    lines.push(format!(
        "    {} value => <String, Object?>{{",
        variant.target_class_name
    ));
    let tag_key_lit = dart_string_literal(tag_key);
    let tag_value_lit = dart_string_literal(&variant.tag);
    let to_json = variant_to_json_expr(variant, "value", serializable_classes);
    if let Some(content_key) = content_key {
        lines.push(format!("      {tag_key_lit}: {tag_value_lit},"));
        lines.push(format!(
            "      {}: {to_json},",
            dart_string_literal(content_key)
        ));
    } else {
        lines.push(format!("      ...{to_json},"));
        lines.push(format!("      {tag_key_lit}: {tag_value_lit},"));
    }
    lines.push("    },".to_owned());
}

/// Renders one variant serialization expression.
fn variant_to_json_expr(
    variant: &SerdeVariantConfigIr,
    value_name: &str,
    serializable_classes: &[&str],
) -> String {
    if contains_symbol(serializable_classes, &variant.target_class_name) {
        format!("_${}ToJson({value_name})", variant.target_class_name)
    } else {
        format!("{value_name}.toJson()")
    }
}

/// Renders one variant deserialization expression.
fn variant_from_json_expr(
    variant: &SerdeVariantConfigIr,
    json_name: &str,
    deserializable_classes: &[&str],
) -> String {
    if contains_symbol(deserializable_classes, &variant.target_class_name) {
        format!("_${}FromJson({json_name})", variant.target_class_name)
    } else {
        format!("{}.fromJson({json_name})", variant.target_class_name)
    }
}

/// Returns true when a generated helper exists for a class name.
fn contains_symbol(symbols: &[&str], name: &str) -> bool {
    symbols.contains(&name)
}

/// Returns tagged sealed serde metadata when generation is supported here.
fn tagged_serde(class: &ClassIr) -> Option<&SerdeClassConfigIr> {
    let serde = class.serde.as_ref()?;
    if serde.tag.is_none() || serde.variants.is_empty() || serde.untagged {
        return None;
    }
    Some(serde)
}
