use dust_dart_emit::dart_string_literal;
use dust_ir::{ClassIr, FieldIr, LibraryIr};
use minijinja::Environment;
use serde::Serialize;

use super::model::{ValidateConfig, field_validations, has_validate_trait};
use super::rule_snippets::rule_line;
use super::type_source::{input_kind, render_type, upper_first};

#[derive(Serialize)]
struct ValidateContext<'a> {
    class_name: &'a str,
    fields: Vec<FieldContext>,
}

#[derive(Clone, Serialize)]
struct FieldContext {
    name: String,
    literal: String,
    helper_name: String,
    input_helper_name: String,
    helper_signature: String,
    input_signature: String,
    type_source: String,
    nullable: bool,
    can_validate_input: bool,
    input_kind: Option<&'static str>,
    parse_error_message: String,
    uses_self: bool,
    configs: Vec<ConfigContext>,
}

#[derive(Clone, Serialize)]
struct ConfigContext {
    nullable: bool,
    required_message: Option<String>,
    rules: Vec<RuleContext>,
}

#[derive(Clone, Serialize)]
struct RuleContext {
    kind: &'static str,
    message: String,
    min: Option<i64>,
    max: Option<String>,
    equal: Option<i64>,
    pattern: Option<String>,
    other: Option<String>,
    custom: Option<String>,
}

pub(crate) struct ValidateEmission {
    pub(crate) mixin_member: String,
    pub(crate) support_type: String,
}

pub(crate) fn emit_validate(_library: &LibraryIr, class: &ClassIr) -> Option<ValidateEmission> {
    if !has_validate_trait(class) {
        return None;
    }

    let fields = render_fields(class);
    let context = ValidateContext {
        class_name: &class.name,
        fields,
    };

    Some(ValidateEmission {
        mixin_member: render_validate_template(&context, "validate_mixin", "validate_mixin.jinja"),
        support_type: render_validate_template(
            &context,
            "validate_support",
            "validate_support.jinja",
        ),
    })
}

fn render_validate_template(
    context: &ValidateContext<'_>,
    name: &str,
    source_name: &str,
) -> String {
    let mut env = Environment::new();
    env.add_function("rule_line", rule_line);
    let source = match source_name {
        "validate_mixin.jinja" => include_str!("templates/validate_mixin.jinja"),
        "validate_support.jinja" => include_str!("templates/validate_support.jinja"),
        _ => unreachable!("unknown validate template"),
    };
    env.add_template(name, source)
        .expect("Dust validate template source must be valid");
    env.get_template(name)
        .expect("Dust validate template must be registered")
        .render(context)
        .expect("Dust validate template context must satisfy template variables")
        .trim_matches('\n')
        .to_owned()
}

fn render_fields(class: &ClassIr) -> Vec<FieldContext> {
    field_validations(class)
        .into_iter()
        .map(|validation| {
            let input_kind = input_kind(&validation.field.ty);
            let uses_self = validation
                .annotations
                .iter()
                .any(|config| config.must_match.is_some());
            let field_name = &validation.field.name;
            let field_type = render_type(&validation.field.ty);
            let helper_name = format!("_validate{}{}", class.name, upper_first(field_name));
            let input_helper_name =
                format!("validate{}{}Input", class.name, upper_first(field_name));
            FieldContext {
                name: validation.field.name.clone(),
                literal: dart_string_literal(&validation.field.name),
                helper_signature: helper_signature(
                    &helper_name,
                    &class.name,
                    field_name,
                    &field_type,
                    uses_self,
                ),
                input_signature: input_signature(&input_helper_name, &class.name, uses_self),
                helper_name,
                input_helper_name,
                type_source: field_type,
                nullable: validation.field.ty.is_nullable(),
                can_validate_input: input_kind.is_some(),
                input_kind,
                parse_error_message: parse_error_message(&validation.annotations),
                uses_self,
                configs: validation
                    .annotations
                    .iter()
                    .map(|config| render_config(validation.field, config))
                    .collect(),
            }
        })
        .collect()
}

fn helper_signature(
    helper_name: &str,
    class_name: &str,
    field_name: &str,
    field_type: &str,
    uses_self: bool,
) -> String {
    if uses_self {
        format!(
            "void {helper_name}(\n  {class_name} self,\n  {field_type} {field_name},\n  List<ValidationError> errors,\n)"
        )
    } else {
        format!("void {helper_name}({field_type} {field_name}, List<ValidationError> errors)")
    }
}

fn input_signature(input_helper_name: &str, class_name: &str, uses_self: bool) -> String {
    if uses_self {
        format!("String? {input_helper_name}(\n  {class_name} self,\n  String? value,\n)")
    } else {
        format!("String? {input_helper_name}(String? value)")
    }
}

fn parse_error_message(configs: &[ValidateConfig]) -> String {
    let message = configs
        .iter()
        .find_map(|config| config.message.as_deref())
        .unwrap_or("Invalid number");
    dart_string_literal(message)
}

fn render_config(field: &FieldIr, config: &ValidateConfig) -> ConfigContext {
    ConfigContext {
        nullable: field.ty.is_nullable(),
        required_message: config
            .required
            .then(|| dart_string_literal(message(config, "Required"))),
        rules: render_rules(config),
    }
}

fn render_rules(config: &ValidateConfig) -> Vec<RuleContext> {
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
            rules.push(rule("length_min", config).with_min(min));
        }
        if let Some(max) = length.max {
            rules.push(rule("length_max", config).with_min(max));
        }
    }
    if let Some(range) = &config.range {
        if let Some(min) = &range.min {
            rules.push(rule("range_min", config).with_max(min.clone()));
        }
        if let Some(max) = &range.max {
            rules.push(rule("range_max", config).with_max(max.clone()));
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

fn rule(kind: &'static str, config: &ValidateConfig) -> RuleContext {
    RuleContext {
        kind,
        message: dart_string_literal(message(config, fallback(kind))),
        min: None,
        max: None,
        equal: None,
        pattern: None,
        other: None,
        custom: None,
    }
}

fn fallback(kind: &str) -> &'static str {
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

fn message<'a>(config: &'a ValidateConfig, fallback: &'a str) -> &'a str {
    config.message.as_deref().unwrap_or(fallback)
}

impl RuleContext {
    fn with_min(mut self, value: i64) -> Self {
        self.min = Some(value);
        self
    }

    fn with_max(mut self, value: String) -> Self {
        self.max = Some(value);
        self
    }

    fn with_equal(mut self, value: i64) -> Self {
        self.equal = Some(value);
        self
    }

    fn with_pattern(mut self, value: &str) -> Self {
        self.pattern = Some(dart_string_literal(value));
        self
    }

    fn with_other(mut self, value: &str) -> Self {
        self.other = Some(value.to_owned());
        self
    }

    fn with_custom(mut self, value: &str) -> Self {
        self.custom = Some(value.to_owned());
        self
    }
}
