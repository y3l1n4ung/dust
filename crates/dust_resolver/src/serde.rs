use dust_diagnostics::Diagnostic;
use dust_ir::{
    ConfigApplicationIr, SerdeClassConfigIr, SerdeEnumVariantConfigIr, SerdeRenameRuleIr,
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
        if config.named_args.len() != config.argument_items().len() {
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
