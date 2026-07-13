use dust_ir::{AnnotationValueIr, ClassIr, ConfigApplicationIr, FieldIr};

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
    if !config.positional_args.is_empty() {
        return None;
    }
    Some(parse_structured_validate_config(config))
}

/// Parses Validate options directly from canonical annotation values.
fn parse_structured_validate_config(config: &ConfigApplicationIr) -> ValidateConfig {
    let mut parsed = ValidateConfig::default();
    for (name, value) in &config.named_args {
        match (name.as_str(), value) {
            ("email", AnnotationValueIr::Bool(value)) => parsed.email = *value,
            ("url", AnnotationValueIr::Bool(value)) => parsed.url = *value,
            (
                "length",
                AnnotationValueIr::Constructor {
                    name,
                    positional_args,
                    named_args,
                },
            ) if name.short == "Length" && positional_args.is_empty() => {
                parsed.length = parse_length(named_args);
            }
            (
                "range",
                AnnotationValueIr::Constructor {
                    name,
                    positional_args,
                    named_args,
                },
            ) if name.short == "Range" && positional_args.is_empty() => {
                parsed.range = parse_range(named_args);
            }
            ("contains", AnnotationValueIr::String(value)) => parsed.contains = Some(value.clone()),
            ("doesNotContain", AnnotationValueIr::String(value)) => {
                parsed.does_not_contain = Some(value.clone());
            }
            ("regex", AnnotationValueIr::String(value)) => parsed.regex = Some(value.clone()),
            ("mustMatch", AnnotationValueIr::String(value)) => {
                parsed.must_match = Some(value.clone());
            }
            ("nested", AnnotationValueIr::Bool(value)) => parsed.nested = *value,
            ("custom", AnnotationValueIr::Member(value)) => {
                parsed.custom = Some(value.source.clone());
            }
            ("required", AnnotationValueIr::Bool(value)) => parsed.required = *value,
            ("message", AnnotationValueIr::String(value)) => parsed.message = Some(value.clone()),
            _ => {}
        }
    }
    parsed
}

/// Parses a `Length(...)` annotation value.
fn parse_length(
    values: &std::collections::BTreeMap<String, AnnotationValueIr>,
) -> Option<LengthRule> {
    let mut rule = length(None, None, None);
    for (key, value) in values {
        let AnnotationValueIr::Number { source, .. } = value else {
            return None;
        };
        let value = source.parse::<i64>().ok()?;
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
fn parse_range(
    values: &std::collections::BTreeMap<String, AnnotationValueIr>,
) -> Option<RangeRule> {
    let mut rule = range(None, None);
    for (key, value) in values {
        let AnnotationValueIr::Number { source, .. } = value else {
            return None;
        };
        match key.as_str() {
            "min" => rule.min = Some(source.clone()),
            "max" => rule.max = Some(source.clone()),
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use dust_ir::{
        AnnotationNumberKindIr, AnnotationValueIr, ConfigApplicationIr, NameIr, SpanIr, SymbolId,
    };
    use dust_text::{FileId, TextRange};

    use super::parse_validate_config;

    #[test]
    fn structured_config_does_not_require_raw_argument_parsing() {
        let span = SpanIr::new(FileId::new(1), TextRange::new(0_u32, 10_u32));
        let config = ConfigApplicationIr::with_arguments(
            SymbolId::new("dust_dart::Validate"),
            Some("(not valid Dart".to_owned()),
            Vec::new(),
            BTreeMap::from([
                ("email".to_owned(), AnnotationValueIr::Bool(true)),
                (
                    "message".to_owned(),
                    AnnotationValueIr::String("bad email".to_owned()),
                ),
                (
                    "length".to_owned(),
                    AnnotationValueIr::Constructor {
                        name: NameIr {
                            source: "Length".to_owned(),
                            short: "Length".to_owned(),
                            prefix: None,
                            span,
                        },
                        positional_args: Vec::new(),
                        named_args: BTreeMap::from([(
                            "min".to_owned(),
                            AnnotationValueIr::Number {
                                source: "2".to_owned(),
                                kind: AnnotationNumberKindIr::Int,
                            },
                        )]),
                    },
                ),
                (
                    "range".to_owned(),
                    AnnotationValueIr::Constructor {
                        name: NameIr {
                            source: "Range".to_owned(),
                            short: "Range".to_owned(),
                            prefix: None,
                            span,
                        },
                        positional_args: Vec::new(),
                        named_args: BTreeMap::from([(
                            "min".to_owned(),
                            AnnotationValueIr::Number {
                                source: "-2".to_owned(),
                                kind: AnnotationNumberKindIr::Int,
                            },
                        )]),
                    },
                ),
            ]),
            span,
        );

        let parsed = parse_validate_config(&config).expect("structured config should parse");

        assert!(parsed.email);
        assert_eq!(parsed.message.as_deref(), Some("bad email"));
        assert_eq!(parsed.length.and_then(|rule| rule.min), Some(2));
        assert_eq!(
            parsed.range.and_then(|rule| rule.min),
            Some("-2".to_owned())
        );
    }
}
