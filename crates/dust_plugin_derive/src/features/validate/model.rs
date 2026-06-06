use dust_ir::{ClassIr, ConfigApplicationIr, FieldIr};
use dust_parser_dart::{AnnotationValue, parse_annotation_named_values};

use crate::features::{VALIDATE_SYMBOL, eq_hash::has_trait};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FieldValidation<'a> {
    pub(crate) field: &'a FieldIr,
    pub(crate) annotations: Vec<ValidateConfig>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct ValidateConfig {
    pub(crate) email: bool,
    pub(crate) url: bool,
    pub(crate) length: Option<LengthRule>,
    pub(crate) range: Option<RangeRule>,
    pub(crate) contains: Option<String>,
    pub(crate) does_not_contain: Option<String>,
    pub(crate) regex: Option<String>,
    pub(crate) must_match: Option<String>,
    pub(crate) nested: bool,
    pub(crate) custom: Option<String>,
    pub(crate) required: bool,
    pub(crate) message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LengthRule {
    pub(crate) min: Option<i64>,
    pub(crate) max: Option<i64>,
    pub(crate) equal: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RangeRule {
    pub(crate) min: Option<String>,
    pub(crate) max: Option<String>,
}

pub(crate) fn has_validate_trait(class: &ClassIr) -> bool {
    has_trait(class, VALIDATE_SYMBOL)
}

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

pub(crate) fn parse_validate_config(config: &ConfigApplicationIr) -> Option<ValidateConfig> {
    let values = parse_annotation_named_values(config.arguments_source.as_deref()?)?;
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

fn is_named(value: &str, expected: &str) -> bool {
    value == expected || value.ends_with(&format!(".{expected}"))
}

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

fn length(min: Option<i64>, max: Option<i64>, equal: Option<i64>) -> LengthRule {
    LengthRule { min, max, equal }
}

fn range(min: Option<&str>, max: Option<&str>) -> RangeRule {
    RangeRule {
        min: min.map(str::to_owned),
        max: max.map(str::to_owned),
    }
}
