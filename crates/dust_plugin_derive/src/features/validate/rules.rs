use dust_dart_emit::{DART_DOUBLE, DART_INT, DART_NUM, DART_STRING};
use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, ConfigApplicationIr, DartFileIr, FieldIr, TypeIr};
use dust_parser_dart::AnnotationValue;

use super::{
    model::{
        ValidateConfig, field_validations, has_validate_trait, parse_config_named_values,
        parse_validate_config,
    },
    type_source::{input_kind, supports_length},
};
use crate::features::{
    VALIDATE_SYMBOL,
    names::{library_declaration_names, upper_first},
};

/// Validates `Validate` derive usage for one class.
pub(crate) fn validate_validate(library: &DartFileIr, class: &ClassIr) -> Vec<Diagnostic> {
    if !has_validate_trait(class) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    validate_public_validator_names(library, class, &mut diagnostics);
    for validation in field_validations(class) {
        for config in &validation.annotations {
            validate_field_config(library, class, validation.field, config, &mut diagnostics);
        }
    }
    for field in &class.fields {
        for config in field
            .configs
            .iter()
            .filter(|config| config.symbol.0 == VALIDATE_SYMBOL)
        {
            validate_config_shape(config, &mut diagnostics);
        }
    }
    diagnostics
}

/// Ensures generated public validator helper names do not collide.
fn validate_public_validator_names(
    library: &DartFileIr,
    class: &ClassIr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let declaration_names = library_declaration_names(library);
    for validation in field_validations(class) {
        if input_kind(&validation.field.ty).is_none() {
            continue;
        }

        let validator_name = format!(
            "validate{}{}Input",
            class.name,
            upper_first(&validation.field.name)
        );
        if declaration_names.contains(&validator_name) {
            diagnostics.push(Diagnostic::error(format!(
                "generated validator `{validator_name}` for `{}.{}` conflicts with an existing top-level declaration",
                class.name, validation.field.name
            )));
        }
    }
}

/// Validates one field's parsed validation config against its type.
fn validate_field_config(
    library: &DartFileIr,
    class: &ClassIr,
    field: &FieldIr,
    config: &ValidateConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if (config.email
        || config.url
        || config.contains.is_some()
        || config.does_not_contain.is_some()
        || config.regex.is_some()
        || config.must_match.is_some())
        && !field.ty.is_named(DART_STRING)
    {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate` string validators on `{}` require `String` or `String?`",
            field.name
        )));
    }
    if config.length.is_some() && !supports_length(&field.ty) {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate(length: ...)` on `{}` requires String, List, Set, or Map",
            field.name
        )));
    }
    if config.range.is_some() && !is_numeric(&field.ty) {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate(range: ...)` on `{}` requires int, double, or num",
            field.name
        )));
    }
    if let Some(other) = &config.must_match {
        validate_must_match(class, field, other, diagnostics);
    }
    if config.nested {
        validate_nested(library, field, diagnostics);
    }
}

/// Validates raw `@Validate(...)` argument shapes.
fn validate_config_shape(config: &ConfigApplicationIr, diagnostics: &mut Vec<Diagnostic>) {
    let Some(values) = parse_config_named_values(config) else {
        diagnostics.push(Diagnostic::error("invalid `@Validate(...)` arguments"));
        return;
    };

    for (name, value) in values {
        match name.as_str() {
            "email" | "url" | "nested" | "required"
                if !matches!(value, AnnotationValue::Bool(_)) =>
            {
                diagnostics.push(Diagnostic::error(format!(
                    "`@Validate({name}: ...)` expects a boolean literal"
                )));
            }
            "length" => validate_length_record(&value, diagnostics),
            "range" => validate_range_record(&value, diagnostics),
            "contains" | "doesNotContain" | "regex" | "mustMatch" | "message"
                if !matches!(value, AnnotationValue::String(_)) =>
            {
                diagnostics.push(Diagnostic::error(format!(
                    "`@Validate({name}: ...)` expects a string literal"
                )));
            }
            "custom" if !matches!(value, AnnotationValue::Member(_)) => {
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
    if parse_validate_config(config).is_none() {
        diagnostics.push(Diagnostic::error("invalid `@Validate(...)` configuration"));
    }
}

/// Validates a `Length(...)` rule payload.
fn validate_length_record(value: &AnnotationValue, diagnostics: &mut Vec<Diagnostic>) {
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
fn validate_range_record(value: &AnnotationValue, diagnostics: &mut Vec<Diagnostic>) {
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
    name: &str,
    constructor: &str,
    value: &'a AnnotationValue,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<&'a [(String, AnnotationValue)]> {
    let AnnotationValue::Constructor {
        name: actual,
        named: values,
    } = value
    else {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate({name}: ...)` expects `{constructor}(...)`"
        )));
        return None;
    };
    if actual != constructor && !actual.ends_with(&format!(".{constructor}")) {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate({name}: ...)` expects `{constructor}(...)`"
        )));
        return None;
    }
    if values.is_empty() {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate({name}: ...)` constructor cannot be empty"
        )));
        return None;
    }
    Some(values)
}

/// Parses and validates an integer rule value.
fn validate_integer(
    option: &str,
    key: &str,
    value: &AnnotationValue,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<i64> {
    let AnnotationValue::Number(value) = value else {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate({option}: ...)` key `{key}` expects an integer literal"
        )));
        return None;
    };
    match value.parse::<i64>() {
        Ok(value) => Some(value),
        Err(_) => {
            diagnostics.push(Diagnostic::error(format!(
                "`@Validate({option}: ...)` key `{key}` expects an integer literal"
            )));
            None
        }
    }
}

/// Parses and validates a numeric rule value.
fn validate_number(
    option: &str,
    key: &str,
    value: &AnnotationValue,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<f64> {
    let AnnotationValue::Number(value) = value else {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate({option}: ...)` key `{key}` expects a numeric literal"
        )));
        return None;
    };
    match value.parse::<f64>() {
        Ok(value) => Some(value),
        Err(_) => {
            diagnostics.push(Diagnostic::error(format!(
                "`@Validate({option}: ...)` key `{key}` expects a numeric literal"
            )));
            None
        }
    }
}

/// Validates that a must-match target exists with matching type.
fn validate_must_match(
    class: &ClassIr,
    field: &FieldIr,
    other: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match class
        .fields
        .iter()
        .find(|candidate| candidate.name == other)
    {
        Some(candidate) if candidate.ty != field.ty => {
            diagnostics.push(Diagnostic::error(format!(
                "`@Validate(mustMatch: '{other}')` type must match `{}`",
                field.name
            )))
        }
        Some(_) => {}
        None => diagnostics.push(Diagnostic::error(format!(
            "`@Validate(mustMatch: '{other}')` references a missing field"
        ))),
    }
}

/// Validates that nested validation targets another `Validate` class.
fn validate_nested(library: &DartFileIr, field: &FieldIr, diagnostics: &mut Vec<Diagnostic>) {
    let Some(name) = field.ty.name() else {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate(nested: true)` on `{}` requires a named type",
            field.name
        )));
        return;
    };
    let Some(target) = library.classes.iter().find(|class| class.name == name) else {
        return;
    };
    if !has_validate_trait(target) {
        diagnostics.push(Diagnostic::error(format!(
            "`@Validate(nested: true)` target `{name}` must derive `Validate()`"
        )));
    }
}

/// Returns true when a type supports range validation.
fn is_numeric(ty: &TypeIr) -> bool {
    ty.is_named(DART_INT) || ty.is_named(DART_DOUBLE) || ty.is_named(DART_NUM)
}
