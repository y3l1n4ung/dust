use dust_ir::{ClassIr, ConfigApplicationIr, FieldIr};
use dust_parser_dart::{AnnotationValue, parse_annotation_named_values};

use crate::features::{VALIDATE_SYMBOL, eq_hash::has_trait};

/// Validation annotations attached to one field.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FieldValidation<'a> {
    /// Source field being validated.
    pub(crate) field: &'a FieldIr,
    /// Parsed validation configs attached to the field.
    pub(crate) annotations: Vec<ValidateConfig>,
}

/// Parsed `@Validate` configuration.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ValidateConfig {
    /// Whether to validate email shape.
    pub(crate) email: bool,
    /// Whether to validate URL shape.
    pub(crate) url: bool,
    /// Optional length rule.
    pub(crate) length: Option<LengthRule>,
    /// Optional numeric range rule.
    pub(crate) range: Option<RangeRule>,
    /// Required substring rule.
    pub(crate) contains: Option<String>,
    /// Forbidden substring rule.
    pub(crate) does_not_contain: Option<String>,
    /// Regular expression rule.
    pub(crate) regex: Option<String>,
    /// Other field that must equal this field.
    pub(crate) must_match: Option<String>,
    /// Whether to call nested validation.
    pub(crate) nested: bool,
    /// Custom validator function reference.
    pub(crate) custom: Option<String>,
    /// Whether nullable values are required.
    pub(crate) required: bool,
    /// Optional custom error message.
    pub(crate) message: Option<String>,
}

/// Parsed `Length` validator rule.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LengthRule {
    /// Minimum accepted length.
    pub(crate) min: Option<i64>,
    /// Maximum accepted length.
    pub(crate) max: Option<i64>,
    /// Exact accepted length.
    pub(crate) equal: Option<i64>,
}

/// Parsed `Range` validator rule.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RangeRule {
    /// Minimum accepted numeric expression.
    pub(crate) min: Option<String>,
    /// Maximum accepted numeric expression.
    pub(crate) max: Option<String>,
}

/// Returns true when a class derives `Validate`.
pub(crate) fn has_validate_trait(class: &ClassIr) -> bool {
    has_trait(class, VALIDATE_SYMBOL)
}

/// Returns parsed validation configs for fields on a class.
pub(crate) fn field_validations(class: &ClassIr) -> Vec<FieldValidation<'_>> {
    class
        .fields
        .iter()
        .filter_map(|field| {
            let annotations = field
                .configs
                .iter()
                .filter(|config| config.symbol.0 == VALIDATE_SYMBOL)
                .filter_map(parse_validate_config)
                .collect::<Vec<_>>();
            (!annotations.is_empty()).then_some(FieldValidation { field, annotations })
        })
        .collect()
}

/// Parses one `@Validate` config application.
pub(crate) fn parse_validate_config(config: &ConfigApplicationIr) -> Option<ValidateConfig> {
    let values = parse_config_named_values(config)?;
    let mut parsed = ValidateConfig::default();
    for (name, value) in values {
        match (name.as_str(), value) {
            ("email", AnnotationValue::Bool(value)) => parsed.email = value,
            ("url", AnnotationValue::Bool(value)) => parsed.url = value,
            ("length", AnnotationValue::Constructor { name, named })
                if is_named(&name, "Length") =>
            {
                parsed.length = parse_length(named);
            }
            ("range", AnnotationValue::Constructor { name, named }) if is_named(&name, "Range") => {
                parsed.range = parse_range(named);
            }
            ("contains", AnnotationValue::String(value)) => parsed.contains = Some(value),
            ("doesNotContain", AnnotationValue::String(value)) => {
                parsed.does_not_contain = Some(value);
            }
            ("regex", AnnotationValue::String(value)) => parsed.regex = Some(value),
            ("mustMatch", AnnotationValue::String(value)) => parsed.must_match = Some(value),
            ("nested", AnnotationValue::Bool(value)) => parsed.nested = value,
            ("custom", AnnotationValue::Member(value)) => parsed.custom = Some(value),
            ("required", AnnotationValue::Bool(value)) => parsed.required = value,
            ("message", AnnotationValue::String(value)) => parsed.message = Some(value),
            _ => {}
        }
    }
    Some(parsed)
}

/// Parses normalized annotation arguments into named values.
pub(crate) fn parse_config_named_values(
    config: &ConfigApplicationIr,
) -> Option<Vec<(String, AnnotationValue)>> {
    let source = format!("({})", config.normalized_arguments()?);
    parse_annotation_named_values(&source)
}

/// Returns true when a constructor/member name matches a simple name.
fn is_named(value: &str, expected: &str) -> bool {
    value == expected || value.ends_with(&format!(".{expected}"))
}

/// Parses a `Length(...)` annotation value.
fn parse_length(values: Vec<(String, AnnotationValue)>) -> Option<LengthRule> {
    let mut rule = length(None, None, None);
    for (key, value) in values {
        let AnnotationValue::Number(value) = value else {
            return None;
        };
        let value = value.parse::<i64>().ok()?;
        match key.as_str() {
            "min" => rule.min = Some(value),
            "max" => rule.max = Some(value),
            "exact" => rule.equal = Some(value),
            _ => return None,
        }
    }
    Some(rule)
}

/// Parses a `Range(...)` annotation value.
fn parse_range(values: Vec<(String, AnnotationValue)>) -> Option<RangeRule> {
    let mut rule = range(None, None);
    for (key, value) in values {
        let AnnotationValue::Number(value) = value else {
            return None;
        };
        match key.as_str() {
            "min" => rule.min = Some(value),
            "max" => rule.max = Some(value),
            _ => return None,
        }
    }
    Some(rule)
}

/// Builds a length rule.
fn length(min: Option<i64>, max: Option<i64>, equal: Option<i64>) -> LengthRule {
    LengthRule { min, max, equal }
}

/// Builds a range rule.
fn range(min: Option<&str>, max: Option<&str>) -> RangeRule {
    RangeRule {
        min: min.map(str::to_owned),
        max: max.map(str::to_owned),
    }
}
