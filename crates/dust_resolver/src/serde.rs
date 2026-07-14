use dust_diagnostics::Diagnostic;
use dust_ir::{
    AnnotationValueIr, ConfigApplicationIr, SerdeClassConfigIr, SerdeEnumVariantConfigIr,
    SerdeFieldConfigIr, SerdeRenameRuleIr,
};

/// Normalizes class-level SerDe configuration after symbol resolution.
pub(crate) fn normalize_class_serde(
    class_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeClassConfigIr> {
    let mut serde = SerdeClassConfigIr::default();
    let mut saw_serde = false;

    for config in configs
        .iter()
        .filter(|config| config.symbol.0 == "dust_dart::SerDe")
    {
        saw_serde = true;
        if !config.positional_args.is_empty() {
            diagnostics.push(Diagnostic::error(
                "SerDe config arguments must use parenthesized named arguments",
            ));
        }

        for key in config.named_args.keys().map(String::as_str) {
            match key {
                "rename" => match config.named_string(key) {
                    Some(value) => serde.rename = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "renameAll" => match config.named_member(key).as_deref().and_then(rename_rule) {
                    Some(value) => serde.rename_all = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses an unknown `SerDe(renameAll: ...)` rule"
                    ))),
                },
                "tag" => match config.named_string(key) {
                    Some(value) => serde.tag = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(tag: ...)` value"
                    ))),
                },
                "content" => match config.named_string(key) {
                    Some(value) => serde.content = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(content: ...)` value"
                    ))),
                },
                "untagged" => match config.named_bool(key) {
                    Some(value) => serde.untagged = value,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-boolean `SerDe(untagged: ...)` value"
                    ))),
                },
                "disallowUnrecognizedKeys" => match config.named_bool(key) {
                    Some(value) => serde.disallow_unrecognized_keys = value,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-boolean `SerDe(disallowUnrecognizedKeys: ...)` value"
                    ))),
                },
                "aliases" | "defaultValue" | "skip" | "skipSerializing"
                | "skipDeserializing" | "using" => diagnostics.push(Diagnostic::error(format!(
                    "class `{class_name}` does not support `SerDe({key}: ...)`"
                ))),
                unknown => diagnostics.push(Diagnostic::warning(format!(
                    "class `{class_name}` uses unknown `SerDe` option `{unknown}`"
                ))),
            }
        }
    }

    if serde.content.is_some() && serde.tag.is_none() {
        diagnostics.push(Diagnostic::error(
            "SerDe(content: ...) requires SerDe(tag: ...)",
        ));
    }
    if serde.tag.is_some() && serde.untagged {
        diagnostics.push(Diagnostic::error(
            "SerDe(tag: ...) cannot be used with SerDe(untagged: true)",
        ));
    }

    saw_serde.then_some(serde)
}

/// Parses the short member of a SerDe rename rule.
fn rename_rule(source: &str) -> Option<SerdeRenameRuleIr> {
    match source.trim().rsplit('.').next()? {
        "lowerCase" => Some(SerdeRenameRuleIr::LowerCase),
        "upperCase" => Some(SerdeRenameRuleIr::UpperCase),
        "pascalCase" => Some(SerdeRenameRuleIr::PascalCase),
        "camelCase" => Some(SerdeRenameRuleIr::CamelCase),
        "snakeCase" => Some(SerdeRenameRuleIr::SnakeCase),
        "screamingSnakeCase" => Some(SerdeRenameRuleIr::ScreamingSnakeCase),
        "kebabCase" => Some(SerdeRenameRuleIr::KebabCase),
        "screamingKebabCase" => Some(SerdeRenameRuleIr::ScreamingKebabCase),
        _ => None,
    }
}

/// Normalizes enum-variant SerDe configuration after symbol resolution.
pub(crate) fn normalize_enum_variant_serde(
    variant_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeEnumVariantConfigIr> {
    let mut serde = SerdeEnumVariantConfigIr::default();
    let mut saw_serde = false;
    for config in configs
        .iter()
        .filter(|config| config.symbol.0 == "dust_dart::SerDe")
    {
        saw_serde = true;
        for key in config.named_args.keys().map(String::as_str) {
            match key {
                "rename" => match config.named_string(key) {
                    Some(value) => serde.rename = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "enum variant `{variant_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "skip" => match config.named_bool(key) {
                    Some(value) => serde.skip = value,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "enum variant `{variant_name}` uses a non-boolean `SerDe(skip: ...)` value"
                    ))),
                },
                unknown => diagnostics.push(Diagnostic::error(format!(
                    "enum variant `{variant_name}` does not support `SerDe({unknown}: ...)`"
                ))),
            }
        }
    }
    saw_serde.then_some(serde)
}

/// Normalizes field-level SerDe configuration after symbol resolution.
pub(crate) fn normalize_field_serde(
    field_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeFieldConfigIr> {
    let mut serde = SerdeFieldConfigIr::default();
    let mut saw_serde = false;
    for config in configs
        .iter()
        .filter(|config| config.symbol.0 == "dust_dart::SerDe")
    {
        saw_serde = true;
        for key in config.named_args.keys().map(String::as_str) {
            let source = config.named_argument_source(key).unwrap_or_default();
            match key {
                "rename" => match config.named_string(key) {
                    Some(value) => serde.rename = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-string `SerDe(rename: ...)` value"))),
                },
                "aliases" => match config.named_string_list(key) {
                    Some(value) => serde.aliases = value,
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-string-list `SerDe(aliases: ...)` value"))),
                },
                "using" => match config.named_argument_value(key) {
                    Some(AnnotationValueIr::Constructor { .. }) => serde.codec_source = Some(source.trim().to_owned()),
                    Some(AnnotationValueIr::Member(name)) if name.prefix.is_some() || name.source.contains('.') || name.short.chars().next().is_some_and(|first| first.is_ascii_lowercase()) => serde.codec_source = Some(source.trim().to_owned()),
                    Some(AnnotationValueIr::Member(_)) => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses suspicious `SerDe(using: ...)` type reference `{}`", source.trim())).with_note(codec_guidance())),
                    _ => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses invalid `SerDe(using: ...)` value `{}`", source.trim())).with_note(codec_guidance())),
                },
                "defaultValue" => {
                    serde.default_value_source = Some(source.trim().to_owned());
                    serde.default_value = config.named_argument_value(key).cloned();
                }
                "skip" => match config.named_bool(key) {
                    Some(true) => { serde.skip_serializing = true; serde.skip_deserializing = true; }
                    Some(false) => {}
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-boolean `SerDe(skip: ...)` value"))),
                },
                "skipSerializing" => match config.named_bool(key) {
                    Some(value) => serde.skip_serializing = value,
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-boolean `SerDe(skipSerializing: ...)` value"))),
                },
                "skipDeserializing" => match config.named_bool(key) {
                    Some(value) => serde.skip_deserializing = value,
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-boolean `SerDe(skipDeserializing: ...)` value"))),
                },
                unsupported @ ("renameAll" | "tag" | "content" | "untagged" | "disallowUnrecognizedKeys") => diagnostics.push(Diagnostic::error(format!("field `{field_name}` does not support `SerDe({unsupported}: ...)`"))),
                unknown => diagnostics.push(Diagnostic::warning(format!("field `{field_name}` uses unknown `SerDe` option `{unknown}`"))),
            }
        }
    }
    saw_serde.then_some(serde)
}

/// Returns guidance for passing a codec instance instead of a type or literal.
fn codec_guidance() -> &'static str {
    "Use a codec object such as `const UnixEpochDateTimeCodec()` or `unixEpochDateTimeCodec`."
}
