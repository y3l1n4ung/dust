use dust_diagnostics::Diagnostic;
use dust_ir::{ConfigApplicationIr, SerdeClassConfigIr, SerdeFieldConfigIr};

use super::serde_parse::{
    parse_bool_literal, parse_serde_arguments, parse_serde_rename_rule, parse_string_list,
    parse_string_literal,
};

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

        for (key, value) in parse_serde_arguments(config.arguments_source.as_deref(), diagnostics) {
            match key {
                "rename" => match parse_string_literal(value) {
                    Some(rename) => serde.rename = Some(rename),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "renameAll" => match parse_serde_rename_rule(value) {
                    Some(rule) => serde.rename_all = Some(rule),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses an unknown `SerDe(renameAll: ...)` rule"
                    ))),
                },
                "disallowUnrecognizedKeys" => match parse_bool_literal(value) {
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

    saw_serde.then_some(serde)
}

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

        for (key, value) in parse_serde_arguments(config.arguments_source.as_deref(), diagnostics) {
            match key {
                "rename" => match parse_string_literal(value) {
                    Some(rename) => serde.rename = Some(rename),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "aliases" => match parse_string_list(value) {
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
                "defaultValue" => serde.default_value_source = Some(value.trim().to_owned()),
                "skip" => match parse_bool_literal(value) {
                    Some(true) => {
                        serde.skip_serializing = true;
                        serde.skip_deserializing = true;
                    }
                    Some(false) => {}
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-boolean `SerDe(skip: ...)` value"
                    ))),
                },
                "skipSerializing" => match parse_bool_literal(value) {
                    Some(flag) => serde.skip_serializing = flag,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-boolean `SerDe(skipSerializing: ...)` value"
                    ))),
                },
                "skipDeserializing" => match parse_bool_literal(value) {
                    Some(flag) => serde.skip_deserializing = flag,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-boolean `SerDe(skipDeserializing: ...)` value"
                    ))),
                },
                "renameAll" | "disallowUnrecognizedKeys" => diagnostics.push(
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

fn is_serde_config(config: &ConfigApplicationIr) -> bool {
    config.symbol.0 == "derive_serde_annotation::SerDe"
}

fn parse_codec_source(
    field_name: &str,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    let source = source.trim();
    if source.is_empty() {
        diagnostics.push(
            Diagnostic::error(format!(
                "field `{field_name}` uses empty `SerDe(using: ...)` value"
            ))
            .with_note(codec_source_guidance()),
        );
        return None;
    }

    if parse_string_literal(source).is_some()
        || parse_bool_literal(source).is_some()
        || source == "null"
        || looks_like_number_literal(source)
        || looks_like_collection_literal(source)
        || looks_like_function_literal(source)
    {
        diagnostics.push(
            Diagnostic::error(format!(
                "field `{field_name}` uses invalid `SerDe(using: ...)` value `{source}`"
            ))
            .with_note(codec_source_guidance()),
        );
        return None;
    }

    if looks_like_bare_type_reference(source) {
        diagnostics.push(
            Diagnostic::error(format!(
                "field `{field_name}` uses suspicious `SerDe(using: ...)` type reference `{source}`"
            ))
            .with_note(codec_source_guidance()),
        );
        return None;
    }

    Some(source.to_owned())
}

fn codec_source_guidance() -> &'static str {
    "Use a codec object such as `const UnixEpochDateTimeCodec()` or `unixEpochDateTimeCodec`."
}

fn looks_like_number_literal(source: &str) -> bool {
    let source = source.trim();
    let Some(first) = source.chars().next() else {
        return false;
    };

    first.is_ascii_digit()
        || ((first == '-' || first == '+')
            && source
                .chars()
                .nth(1)
                .is_some_and(|next| next.is_ascii_digit()))
}

fn looks_like_collection_literal(source: &str) -> bool {
    let source = source.trim();
    (source.starts_with('[') && source.ends_with(']'))
        || (source.starts_with('{') && source.ends_with('}'))
}

fn looks_like_function_literal(source: &str) -> bool {
    source.contains("=>")
}

fn looks_like_bare_type_reference(source: &str) -> bool {
    let source = source.trim();
    !source.contains('(')
        && !source.contains('.')
        && source
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
}
