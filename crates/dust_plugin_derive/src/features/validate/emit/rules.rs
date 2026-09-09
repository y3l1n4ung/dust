//! Turning one validation config into the rules a generated validator runs.

use super::*;

/// Builds a template context for one validation config.
pub(super) fn render_config(field: &FieldIr, config: &ValidateConfig) -> ConfigContext {
    ConfigContext {
        nullable: field.ty.is_nullable(),
        required_message: config
            .required
            .then(|| dart_string_literal(message(config, "Required"))),
        rules: render_rules(config),
    }
}

/// Builds all rule contexts for a validation config.
pub(super) fn render_rules(config: &ValidateConfig) -> Vec<RuleContext> {
    let mut rules = Vec::new();
    if config.email {
        rules.push(rule("email", config));
    }
    if config.url {
        rules.push(rule("url", config));
    }
    if let Some(length) = &config.length {
        if let Some(equal) = length.equal {
            rules.push(rule("length_equal", config).with_equal(equal));
        }
        if let Some(min) = length.min {
            rules.push(rule("length_min", config).with_int_value(min));
        }
        if let Some(max) = length.max {
            rules.push(rule("length_max", config).with_int_value(max));
        }
    }
    if let Some(range) = &config.range {
        if let Some(min) = &range.min {
            rules.push(rule("range_min", config).with_number_value(min.clone()));
        }
        if let Some(max) = &range.max {
            rules.push(rule("range_max", config).with_number_value(max.clone()));
        }
    }
    if let Some(pattern) = &config.contains {
        rules.push(rule("contains", config).with_pattern(pattern));
    }
    if let Some(pattern) = &config.does_not_contain {
        rules.push(rule("does_not_contain", config).with_pattern(pattern));
    }
    if let Some(pattern) = &config.regex {
        rules.push(rule("regex", config).with_pattern(pattern));
    }
    if let Some(other) = &config.must_match {
        rules.push(rule("must_match", config).with_other(other));
    }
    if config.nested {
        rules.push(rule("nested", config));
    }
    if let Some(custom) = &config.custom {
        rules.push(rule("custom", config).with_custom(custom));
    }
    rules
}

/// Builds a base rule context for a rule kind.
pub(super) fn rule(kind: &'static str, config: &ValidateConfig) -> RuleContext {
    RuleContext {
        kind,
        message: dart_string_literal(message(config, fallback(kind))),
        int_value: None,
        number_value: None,
        equal: None,
        pattern: None,
        other: None,
        custom: None,
    }
}

/// Returns the fallback message for a generated validation rule.
pub(super) fn fallback(kind: &str) -> &'static str {
    match kind {
        "email" => "Invalid email",
        "url" => "Invalid URL",
        "length_equal" => "Invalid length",
        "length_min" => "Too short",
        "length_max" => "Too long",
        "range_min" => "Too small",
        "range_max" => "Too large",
        "contains" => "Missing required text",
        "does_not_contain" => "Contains forbidden text",
        "regex" => "Invalid format",
        "must_match" => "Fields do not match",
        _ => "Invalid value",
    }
}

/// Returns the configured message or the rule fallback.
pub(super) fn message<'a>(config: &'a ValidateConfig, fallback: &'a str) -> &'a str {
    config.message.as_deref().unwrap_or(fallback)
}

impl RuleContext {
    /// Attaches an integer value to a rule context.
    fn with_int_value(mut self, value: i64) -> Self {
        self.int_value = Some(value);
        self
    }

    /// Attaches a numeric source value to a rule context.
    fn with_number_value(mut self, value: String) -> Self {
        self.number_value = Some(value);
        self
    }

    /// Attaches an exact length value to a rule context.
    fn with_equal(mut self, value: i64) -> Self {
        self.equal = Some(value);
        self
    }

    /// Attaches a Dart string literal pattern to a rule context.
    fn with_pattern(mut self, value: &str) -> Self {
        self.pattern = Some(dart_string_literal(value));
        self
    }

    /// Attaches another field name to a must-match rule context.
    fn with_other(mut self, value: &str) -> Self {
        self.other = Some(value.to_owned());
        self
    }

    /// Attaches a custom validator function to a rule context.
    fn with_custom(mut self, value: &str) -> Self {
        self.custom = Some(value.to_owned());
        self
    }
}
