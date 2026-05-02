use dust_diagnostics::Diagnostic;
use dust_ir::SerdeRenameRuleIr;

use super::parse_support::{split_top_level_items, split_top_level_once};

pub(crate) fn parse_serde_arguments<'a>(
    source: Option<&'a str>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(&'a str, &'a str)> {
    let Some(source) = source.map(str::trim).filter(|source| !source.is_empty()) else {
        return Vec::new();
    };

    let Some(inner) = source
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
    else {
        diagnostics.push(Diagnostic::error(
            "SerDe config arguments must use parenthesized named arguments",
        ));
        return Vec::new();
    };

    let inner = inner.trim();
    if inner.is_empty() {
        return Vec::new();
    }

    let mut arguments = Vec::new();
    for item in split_top_level_items(inner) {
        if let Some((key, value)) = split_named_argument(item) {
            arguments.push((key.trim(), value.trim()));
        } else {
            diagnostics.push(Diagnostic::error(format!(
                "could not parse `SerDe` argument `{item}`"
            )));
        }
    }

    arguments
}

pub(crate) fn parse_string_literal(source: &str) -> Option<String> {
    let source = source.trim();
    let first = source.chars().next()?;
    let last = source.chars().next_back()?;
    if source.len() < 2 || first != last || !matches!(first, '\'' | '"') {
        return None;
    }

    Some(source[1..source.len() - 1].to_owned())
}

pub(crate) fn parse_bool_literal(source: &str) -> Option<bool> {
    match source.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub(crate) fn parse_string_list(source: &str) -> Option<Vec<String>> {
    let source = source.trim();
    let inner = source.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }

    split_top_level_items(inner)
        .into_iter()
        .map(parse_string_literal)
        .collect()
}

pub(crate) fn parse_serde_rename_rule(source: &str) -> Option<SerdeRenameRuleIr> {
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

fn split_named_argument(source: &str) -> Option<(&str, &str)> {
    split_top_level_once(source, ':')
}

pub(crate) fn parse_codec_source(
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
