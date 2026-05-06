use dust_diagnostics::Diagnostic;
use dust_ir::{MethodParamIr, SpanIr};

use crate::plugin::util::{config_name, label, split_top_level_items, split_top_level_once};

pub(crate) fn parse_required_string_argument(
    param: &MethodParamIr,
    annotation_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    parse_optional_string_argument(param, annotation_name, diagnostics).or_else(|| {
        diagnostics.push(
            Diagnostic::error(format!(
                "`@{annotation_name}` on parameter `{}` requires a string argument",
                param.name
            ))
            .with_label(label(
                param.span,
                "add a quoted key argument to this HTTP parameter annotation",
            )),
        );
        None
    })
}

pub(crate) fn parse_optional_string_argument(
    param: &MethodParamIr,
    annotation_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    let config = param
        .configs
        .iter()
        .find(|config| config_name(&config.symbol.0) == annotation_name)?;
    parse_single_string_argument(
        config.arguments_source.as_deref(),
        diagnostics,
        config.span,
        annotation_name,
    )
}

pub(crate) fn parse_single_string_argument(
    source: Option<&str>,
    diagnostics: &mut Vec<Diagnostic>,
    span: SpanIr,
    label_name: &str,
) -> Option<String> {
    let inner = parse_parenthesized_inner(source, diagnostics, span, label_name)?;
    if inner.trim().is_empty() {
        return None;
    }

    match parse_string_literal(inner.trim()) {
        Some(value) => Some(value),
        None => {
            diagnostics.push(
                Diagnostic::error(format!("`{label_name}` expects a quoted string argument"))
                    .with_label(label(span, "use a quoted string literal here")),
            );
            None
        }
    }
}

pub(crate) fn parse_single_map_argument(
    source: Option<&str>,
    diagnostics: &mut Vec<Diagnostic>,
    span: SpanIr,
    label_name: &str,
) -> Vec<(String, String)> {
    let Some(inner) = parse_parenthesized_inner(source, diagnostics, span, label_name) else {
        return Vec::new();
    };
    if inner.trim().is_empty() {
        return Vec::new();
    }
    parse_string_map(inner.trim(), diagnostics, span, label_name)
}

pub(crate) fn parse_named_arguments<'a>(
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
            "annotation arguments must use parenthesized syntax",
        ));
        return Vec::new();
    };

    let inner = inner.trim();
    if inner.is_empty() {
        return Vec::new();
    }

    let mut arguments = Vec::new();
    for item in split_top_level_items(inner) {
        if let Some((key, value)) = split_top_level_once(item, ':') {
            arguments.push((key.trim(), value.trim()));
        } else {
            diagnostics.push(Diagnostic::error(format!(
                "could not parse annotation argument `{item}`"
            )));
        }
    }

    arguments
}

pub(crate) fn parse_string_map(
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
    span: SpanIr,
    label_name: &str,
) -> Vec<(String, String)> {
    let Some(inner) = source
        .strip_prefix('{')
        .and_then(|inner| inner.strip_suffix('}'))
    else {
        diagnostics.push(
            Diagnostic::error(format!("`{label_name}` expects a map literal"))
                .with_label(label(span, "use a `{ 'key': 'value' }` literal here")),
        );
        return Vec::new();
    };

    let inner = inner.trim();
    if inner.is_empty() {
        return Vec::new();
    }

    let mut values = Vec::new();
    for item in split_top_level_items(inner) {
        let Some((raw_key, raw_value)) = split_top_level_once(item, ':') else {
            diagnostics.push(
                Diagnostic::error(format!("could not parse map entry `{item}`"))
                    .with_label(label(span, "use `'<key>': '<value>'` entries")),
            );
            continue;
        };
        let Some(key) = parse_string_literal(raw_key.trim()) else {
            diagnostics.push(
                Diagnostic::error("map keys must be quoted string literals")
                    .with_label(label(span, "quote this header key")),
            );
            continue;
        };
        let Some(value) = parse_string_literal(raw_value.trim()) else {
            diagnostics.push(
                Diagnostic::error("map values must be quoted string literals")
                    .with_label(label(span, "quote this header value")),
            );
            continue;
        };
        values.push((key, value));
    }

    values
}

fn parse_parenthesized_inner<'a>(
    source: Option<&'a str>,
    diagnostics: &mut Vec<Diagnostic>,
    span: SpanIr,
    label_name: &str,
) -> Option<&'a str> {
    let source = source?;
    let source = source.trim();
    let Some(inner) = source
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
    else {
        diagnostics.push(
            Diagnostic::error(format!("`{label_name}` arguments must be parenthesized"))
                .with_label(label(span, "wrap annotation arguments in `(...)`")),
        );
        return None;
    };
    Some(inner.trim())
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

pub(crate) fn parse_enum_variant(source: &str) -> Option<&str> {
    source.trim().rsplit('.').next()
}
