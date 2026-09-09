use dust_dart_emit::dart_string_literal;
use dust_ir::{ClassIr, DartFileIr, FieldIr};
use minijinja::Environment;
use serde::Serialize;

use super::model::{ValidateConfig, field_validations, has_validate_trait};
use super::rule_snippets::rule_line;
use super::type_source::{input_kind, render_type};
use crate::features::names::{NameAllocator, library_declaration_names, upper_first};

/// Turning one validation config into the rules a generated validator runs.
mod rules;
use self::rules::*;

/// Rendering the per-field context a validator template reads.
mod fields;
use self::fields::*;

/// Template context for a class validation extension.
#[derive(Serialize)]
struct ValidateContext<'a> {
    /// Source class name.
    class_name: &'a str,
    /// Generated private validation extension name.
    extension_name: String,
    /// Local receiver variable name.
    self_name: String,
    /// Local validation error list variable name.
    errors_name: String,
    /// Local validation result variable name.
    result_name: String,
    /// Local error list name used by `validateOrThrow`.
    invalid_errors_name: String,
    /// Field validation contexts.
    fields: Vec<FieldContext>,
}

/// Template context for one validated field.
#[derive(Clone, Serialize)]
struct FieldContext {
    /// Field name.
    name: String,
    /// Dart string literal for the field name.
    literal: String,
    /// Local field value variable name.
    value_name: String,
    /// Local receiver variable name.
    self_name: String,
    /// Local validation error list variable name.
    errors_name: String,
    /// Local parsed Flutter form input variable name.
    parsed_name: String,
    /// Local nested validation result variable name.
    nested_validation_name: String,
    /// Local nested errors variable name.
    nested_errors_name: String,
    /// Local nested error loop variable name.
    nested_error_name: String,
    /// Local custom validator result variable name.
    custom_error_name: String,
    /// Private validation helper method name.
    helper_name: String,
    /// Private Flutter form input helper method name.
    input_helper_name: String,
    /// Public top-level Flutter form validator helper name.
    public_input_helper_name: String,
    /// Private validation helper signature.
    helper_signature: String,
    /// Private input helper signature.
    input_signature: String,
    /// Public input helper signature.
    public_input_signature: String,
    /// Rendered Dart field type source.
    type_source: String,
    /// Whether the field type is nullable.
    nullable: bool,
    /// Whether Flutter form input validation can be generated.
    can_validate_input: bool,
    /// Parser kind for Flutter form input validation.
    input_kind: Option<&'static str>,
    /// Error message used when parsing input fails.
    parse_error_message: String,
    /// Whether validation rules need the receiver.
    uses_self: bool,
    /// Parsed validation configs for the field.
    configs: Vec<ConfigContext>,
}

/// Template context for one `@Validate` config.
#[derive(Clone, Serialize)]
struct ConfigContext {
    /// Whether the field is nullable.
    nullable: bool,
    /// Optional required-field error message.
    required_message: Option<String>,
    /// Rule contexts rendered for this config.
    rules: Vec<RuleContext>,
}

/// Template context for one validation rule.
#[derive(Clone, Serialize)]
struct RuleContext {
    /// Rule kind consumed by the rule snippet renderer.
    kind: &'static str,
    /// Dart string literal for the error message.
    message: String,
    /// Integer rule value.
    int_value: Option<i64>,
    /// Numeric rule value source.
    number_value: Option<String>,
    /// Exact length value.
    equal: Option<i64>,
    /// Dart string literal pattern.
    pattern: Option<String>,
    /// Other field name for must-match rules.
    other: Option<String>,
    /// Custom validator function source.
    custom: Option<String>,
}

/// Generated validation mixin member and support type source.
pub(crate) struct ValidateEmission {
    /// Mixin member inserted into the source class mixin.
    pub(crate) mixin_member: String,
    /// Private extension and public validator helper source.
    pub(crate) support_type: String,
}

/// Emits validation support for a class that derives `Validate`.
pub(crate) fn emit_validate(
    library: &DartFileIr,
    class: &ClassIr,
    emit_form_helpers: bool,
) -> Option<ValidateEmission> {
    if !has_validate_trait(class) {
        return None;
    }

    let mut allocator = NameAllocator::new(library_declaration_names(library));
    let extension_name = allocator.allocate(format!("_{}Validation", class.name));
    let mut method_allocator = NameAllocator::new(std::iter::empty::<&str>());
    let self_name = method_allocator.allocate("self");
    let errors_name = method_allocator.allocate("errors");
    let result_name = method_allocator.allocate("result");
    let mut throw_allocator = NameAllocator::new(std::iter::empty::<&str>());
    let invalid_errors_name = throw_allocator.allocate("errors");
    let fields = render_fields(class, emit_form_helpers);
    let context = ValidateContext {
        class_name: &class.name,
        extension_name,
        self_name,
        errors_name,
        result_name,
        invalid_errors_name,
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

/// Renders a validation template with the rule-line helper registered.
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
    let rendered = env
        .get_template(name)
        .expect("Dust validate template must be registered")
        .render(context)
        .expect("Dust validate template context must satisfy template variables")
        .trim_matches('\n')
        .to_owned();
    collapse_excess_blank_lines(rendered)
}

/// Collapses accidental triple blank lines after template rendering.
fn collapse_excess_blank_lines(mut value: String) -> String {
    while value.contains("\n\n\n") {
        value = value.replace("\n\n\n", "\n\n");
    }
    value
}

/// Renders the private field validation helper signature.
fn helper_signature(
    helper_name: &str,
    class_name: &str,
    self_name: &str,
    value_name: &str,
    errors_name: &str,
    field_type: &str,
    uses_self: bool,
) -> String {
    if uses_self {
        format!(
            "static void {helper_name}(\n    {class_name} {self_name},\n    {field_type} {value_name},\n    List<ValidationError> {errors_name},\n  )"
        )
    } else {
        format!(
            "static void {helper_name}({field_type} {value_name}, List<ValidationError> {errors_name})"
        )
    }
}

/// Renders the private Flutter form input helper signature.
fn input_signature(
    input_helper_name: &str,
    class_name: &str,
    self_name: &str,
    uses_self: bool,
) -> String {
    if uses_self {
        format!(
            "static String? {input_helper_name}(\n    {class_name} {self_name},\n    String? value,\n  )"
        )
    } else {
        format!("static String? {input_helper_name}(String? value)")
    }
}

/// Renders the public Flutter form validator helper signature.
fn public_input_signature(
    public_input_helper_name: &str,
    class_name: &str,
    uses_self: bool,
) -> String {
    if uses_self {
        format!("String? {public_input_helper_name}(\n  {class_name} self,\n  String? value,\n)")
    } else {
        format!("String? {public_input_helper_name}(String? value)")
    }
}

/// Renders the parse-error message for numeric Flutter form inputs.
fn parse_error_message(configs: &[ValidateConfig]) -> String {
    let message = configs
        .iter()
        .find_map(|config| config.message.as_deref())
        .unwrap_or("Invalid number");
    dart_string_literal(message)
}
