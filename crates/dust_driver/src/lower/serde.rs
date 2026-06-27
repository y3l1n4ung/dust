use dust_diagnostics::Diagnostic;
use dust_ir::{ConfigApplicationIr, SerdeClassConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr};

use super::serde_parse::{parse_codec_source, parse_serde_rename_rule};

/// Lowers class-level `@SerDe` options.
pub(crate) fn lower_class_serde_config(
    class_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeClassConfigIr> {
    let mut serde = SerdeClassConfigIr::default();
    let mut saw_serde = false;

    for config in configs {
        if !is_serde_config(config) {
            continue;
        }
        saw_serde = true;

        for (key, _) in serde_named_arguments(config, diagnostics) {
            match key {
                "rename" => match config.named_string("rename") {
                    Some(rename) => serde.rename = Some(rename),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "renameAll" => match config
                    .named_member("renameAll")
                    .as_deref()
                    .and_then(parse_serde_rename_rule)
                {
                    Some(rule) => serde.rename_all = Some(rule),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses an unknown `SerDe(renameAll: ...)` rule"
                    ))),
                },
                "tag" => match config.named_string("tag") {
                    Some(tag) => serde.tag = Some(tag),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(tag: ...)` value"
                    ))),
                },
                "content" => match config.named_string("content") {
                    Some(content) => serde.content = Some(content),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(content: ...)` value"
                    ))),
                },
                "untagged" => match config.named_bool("untagged") {
                    Some(flag) => serde.untagged = flag,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-boolean `SerDe(untagged: ...)` value"
                    ))),
                },
                "disallowUnrecognizedKeys" => match config.named_bool("disallowUnrecognizedKeys") {
                    Some(flag) => serde.disallow_unrecognized_keys = flag,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-boolean `SerDe(disallowUnrecognizedKeys: ...)` value"
                    ))),
                },
                "aliases"
                | "defaultValue"
                | "skip"
                | "skipSerializing"
                | "skipDeserializing"
                | "using" => diagnostics.push(Diagnostic::error(format!(
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

/// Resolves one sealed variant tag from constructor-level `@SerDe` options.
pub(crate) fn lower_variant_serde_tag(
    variant_name: &str,
    configs: &[ConfigApplicationIr],
    rename_all: Option<SerdeRenameRuleIr>,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let mut rename = None;

    for config in configs {
        if !is_serde_config(config) {
            continue;
        }

        for (key, _) in serde_named_arguments(config, diagnostics) {
            match key {
                "rename" => match config.named_string("rename") {
                    Some(value) => rename = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "variant `{variant_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "renameAll"
                | "tag"
                | "content"
                | "untagged"
                | "disallowUnrecognizedKeys"
                | "aliases"
                | "defaultValue"
                | "skip"
                | "skipSerializing"
                | "skipDeserializing"
                | "using" => diagnostics.push(Diagnostic::error(format!(
                    "variant `{variant_name}` does not support `SerDe({key}: ...)`"
                ))),
                unknown => diagnostics.push(Diagnostic::warning(format!(
                    "variant `{variant_name}` uses unknown `SerDe` option `{unknown}`"
                ))),
            }
        }
    }

    rename.unwrap_or_else(|| match rename_all {
        Some(rule) => dust_dart_emit::apply_rename_rule(variant_name, rule),
        None => variant_name.to_owned(),
    })
}

/// Lowers field-level `@SerDe` options.
pub(crate) fn lower_field_serde_config(
    field_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeFieldConfigIr> {
    let mut serde = SerdeFieldConfigIr::default();
    let mut saw_serde = false;

    for config in configs {
        if !is_serde_config(config) {
            continue;
        }
        saw_serde = true;

        for (key, value) in serde_named_arguments(config, diagnostics) {
            match key {
                "rename" => match config.named_string("rename") {
                    Some(rename) => serde.rename = Some(rename),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "aliases" => match config.named_string_list("aliases") {
                    Some(aliases) => serde.aliases = aliases,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-string-list `SerDe(aliases: ...)` value"
                    ))),
                },
                "using" => {
                    if let Some(codec_source) = parse_codec_source(field_name, value, diagnostics) {
                        serde.codec_source = Some(codec_source);
                    }
                }
                "defaultValue" => {
                    serde.default_value_source = Some(value.trim().to_owned());
                    serde.default_value = config.named_argument_value("defaultValue").cloned();
                }
                "skip" => match config.named_bool("skip") {
                    Some(true) => {
                        serde.skip_serializing = true;
                        serde.skip_deserializing = true;
                    }
                    Some(false) => {}
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-boolean `SerDe(skip: ...)` value"
                    ))),
                },
                "skipSerializing" => match config.named_bool("skipSerializing") {
                    Some(flag) => serde.skip_serializing = flag,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-boolean `SerDe(skipSerializing: ...)` value"
                    ))),
                },
                "skipDeserializing" => match config.named_bool("skipDeserializing") {
                    Some(flag) => serde.skip_deserializing = flag,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-boolean `SerDe(skipDeserializing: ...)` value"
                    ))),
                },
                "renameAll" | "tag" | "content" | "untagged" | "disallowUnrecognizedKeys" => diagnostics.push(
                    Diagnostic::error(format!(
                        "field `{field_name}` does not support `SerDe({key}: ...)`"
                    )),
                ),
                unknown => diagnostics.push(Diagnostic::warning(format!(
                    "field `{field_name}` uses unknown `SerDe` option `{unknown}`"
                ))),
            }
        }
    }

    saw_serde.then_some(serde)
}

/// Returns whether a config application is the Dust SerDe annotation.
fn is_serde_config(config: &ConfigApplicationIr) -> bool {
    config.symbol.0 == "dust_dart::SerDe"
}

/// Returns named SerDe arguments and reports malformed positional usage.
fn serde_named_arguments<'a>(
    config: &'a ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(&'a str, &'a str)> {
    let items = config.argument_items();
    let named = config.named_arguments();
    if named.len() != items.len() {
        diagnostics.push(Diagnostic::error(
            "SerDe config arguments must use parenthesized named arguments",
        ));
    }
    named
}
