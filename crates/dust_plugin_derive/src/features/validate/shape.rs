use std::collections::BTreeMap;

use dust_diagnostics::Diagnostic;
use dust_ir::{AnnotationNumberKindIr, AnnotationValueIr, ConfigApplicationIr};

/// Validates structured `@Validate(...)` argument shapes.
pub(super) fn validate_config_shape(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !config.positional_args.is_empty() {
        diagnostics.push(Diagnostic::error("invalid `@Validate(...)` arguments"));
        return;
    }

    for (name, value) in &config.named_args {
        match name.as_str() {
            "email" | "url" | "nested" | "required"
                if !matches!(value, AnnotationValueIr::Bool(_)) =>
            {
                diagnostics.push(Diagnostic::error(format!(
                    "`@Validate({name}: ...)` expects a boolean literal"
                )));
            }
            "length" => validate_length_record(value, diagnostics),
            "range" => validate_range_record(value, diagnostics),
            "contains" | "doesNotContain" | "regex" | "mustMatch" | "message"
                if !matches!(value, AnnotationValueIr::String(_)) =>
            {
                diagnostics.push(Diagnostic::error(format!(
                    "`@Validate({name}: ...)` expects a string literal"
                )));
            }
            "custom" if !matches!(value, AnnotationValueIr::Member(_)) => {
                diagnostics.push(Diagnostic::error(
                    "`@Validate(custom: ...)` expects a function reference",
                ));
            }
            "email" | "url" | "nested" | "required" | "contains" | "doesNotContain" | "regex"
            | "mustMatch" | "message" | "custom" => {}
            _ => diagnostics.push(Diagnostic::error(format!(
                "unknown `@Validate` option `{name}`"
            ))),
        }
    }
}

/// Validates a `Length(...)` rule payload.
fn validate_length_record(value: &AnnotationValueIr, diagnostics: &mut Vec<Diagnostic>) {
    let Some(values) = validate_constructor_shape("length", "Length", value, diagnostics) else {
        return;
    };
    let mut min = None;
    let mut max = None;
    let mut exact = None;
    for (key, value) in values {
        match key.as_str() {
            "min" => min = validate_integer("length", key, value, diagnostics),
            "max" => max = validate_integer("length", key, value, diagnostics),
            "exact" => exact = validate_integer("length", key, value, diagnostics),
            _ => diagnostics.push(Diagnostic::error(format!(
                "unknown `@Validate(length: Length(...))` key `{key}`"
            ))),
        }
    }
    if exact.is_some() && (min.is_some() || max.is_some()) {
        diagnostics.push(Diagnostic::error(
            "`@Validate(length: ...)` cannot combine `exact` with `min` or `max`",
        ));
    }
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        diagnostics.push(Diagnostic::error(
            "`@Validate(length: ...)` requires `min` to be <= `max`",
        ));
    }
}

/// Validates a `Range(...)` rule payload.
fn validate_range_record(value: &AnnotationValueIr, diagnostics: &mut Vec<Diagnostic>) {
    let Some(values) = validate_constructor_shape("range", "Range", value, diagnostics) else {
        return;
    };
    let mut min = None;
    let mut max = None;
    for (key, value) in values {
        match key.as_str() {
            "min" => min = validate_number("range", key, value, diagnostics),
            "max" => max = validate_number("range", key, value, diagnostics),
            _ => diagnostics.push(Diagnostic::error(format!(
                "unknown `@Validate(range: Range(...))` key `{key}`"
            ))),
        }
    }
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        diagnostics.push(Diagnostic::error(
            "`@Validate(range: ...)` requires `min` to be <= `max`",
        ));
    }
}

/// Validates a constructor-shaped annotation argument.
fn validate_constructor_shape<'a>(
    option: &str,
    constructor: &str,
    value: &'a AnnotationValueIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<&'a BTreeMap<String, AnnotationValueIr>> {
    let AnnotationValueIr::Constructor {
        name,
        positional_args,
        named_args,
    } = value
    else {
        diagnostics.push(expected_constructor(option, constructor));
        return None;
    };
    if name.short != constructor || !positional_args.is_empty() {
        diagnostics.push(expected_constructor(option, constructor));
        return None;
    }
    if named_args.is_empty() {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate({option}: ...)` constructor cannot be empty"
        )));
        return None;
    }
    Some(named_args)
}

/// Creates the standard wrong-constructor diagnostic.
fn expected_constructor(option: &str, constructor: &str) -> Diagnostic {
    Diagnostic::error(format!(
        "`@Validate({option}: ...)` expects `{constructor}(...)`"
    ))
}

/// Parses and validates an integer rule value.
fn validate_integer(
    option: &str,
    key: &str,
    value: &AnnotationValueIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<i64> {
    let AnnotationValueIr::Number {
        source,
        kind: AnnotationNumberKindIr::Int,
    } = value
    else {
        diagnostics.push(expected_number(option, key, "integer"));
        return None;
    };
    source.parse::<i64>().ok().or_else(|| {
        diagnostics.push(expected_number(option, key, "integer"));
        None
    })
}

/// Parses and validates a numeric rule value.
fn validate_number(
    option: &str,
    key: &str,
    value: &AnnotationValueIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<f64> {
    let AnnotationValueIr::Number { source, .. } = value else {
        diagnostics.push(expected_number(option, key, "numeric"));
        return None;
    };
    source.parse::<f64>().ok().or_else(|| {
        diagnostics.push(expected_number(option, key, "numeric"));
        None
    })
}

/// Creates a numeric-literal diagnostic.
fn expected_number(option: &str, key: &str, kind: &str) -> Diagnostic {
    let article = if kind == "integer" { "an" } else { "a" };
    Diagnostic::error(format!(
        "`@Validate({option}: ...)` key `{key}` expects {article} {kind} literal"
    ))
}
