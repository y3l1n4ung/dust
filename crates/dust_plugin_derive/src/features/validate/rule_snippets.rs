use minijinja::Value;

pub(crate) fn rule_line(field: Value, rule: Value, indent: String) -> String {
    let name = field.get_attr("name").unwrap().to_string();
    let value_name = field.get_attr("value_name").unwrap().to_string();
    let self_name = field.get_attr("self_name").unwrap().to_string();
    let errors_name = field.get_attr("errors_name").unwrap().to_string();
    let nested_validation_name = field
        .get_attr("nested_validation_name")
        .unwrap()
        .to_string();
    let nested_errors_name = field.get_attr("nested_errors_name").unwrap().to_string();
    let nested_error_name = field.get_attr("nested_error_name").unwrap().to_string();
    let custom_error_name = field.get_attr("custom_error_name").unwrap().to_string();
    let literal = field.get_attr("literal").unwrap().to_string();
    let kind = rule.get_attr("kind").unwrap().to_string();
    let message = rule.get_attr("message").unwrap().to_string();
    match kind.as_str() {
        "email" => simple_helper(
            &indent,
            &value_name,
            &errors_name,
            &literal,
            &message,
            "isEmail",
        ),
        "url" => simple_helper(
            &indent,
            &value_name,
            &errors_name,
            &literal,
            &message,
            "isUrl",
        ),
        "length_equal" => compare(
            &indent,
            format!("{value_name}.length != {}", rule.get_attr("equal").unwrap()),
            &errors_name,
            &literal,
            &message,
        ),
        "length_min" => compare(
            &indent,
            format!(
                "{value_name}.length < {}",
                rule.get_attr("int_value").unwrap()
            ),
            &errors_name,
            &literal,
            &message,
        ),
        "length_max" => compare(
            &indent,
            format!(
                "{value_name}.length > {}",
                rule.get_attr("int_value").unwrap()
            ),
            &errors_name,
            &literal,
            &message,
        ),
        "range_min" => compare(
            &indent,
            format!("{value_name} < {}", rule.get_attr("number_value").unwrap()),
            &errors_name,
            &literal,
            &message,
        ),
        "range_max" => compare(
            &indent,
            format!("{value_name} > {}", rule.get_attr("number_value").unwrap()),
            &errors_name,
            &literal,
            &message,
        ),
        "contains" => string_contains(
            &indent,
            &value_name,
            &errors_name,
            &rule,
            false,
            &literal,
            &message,
        ),
        "does_not_contain" => string_contains(
            &indent,
            &value_name,
            &errors_name,
            &rule,
            true,
            &literal,
            &message,
        ),
        "regex" => regex(
            &indent,
            &value_name,
            &errors_name,
            &rule,
            &literal,
            &message,
        ),
        "must_match" => must_match(
            &indent,
            &value_name,
            &self_name,
            &errors_name,
            &rule,
            &literal,
            &message,
        ),
        "nested" => nested(
            &indent,
            &name,
            &value_name,
            &errors_name,
            &nested_validation_name,
            &nested_errors_name,
            &nested_error_name,
        ),
        "custom" => custom(
            &indent,
            &value_name,
            &errors_name,
            &custom_error_name,
            &rule,
        ),
        _ => String::new(),
    }
}

fn simple_helper(
    indent: &str,
    name: &str,
    errors_name: &str,
    literal: &str,
    message: &str,
    helper: &str,
) -> String {
    block(
        indent,
        [
            format!("if (!ValidationHelper.{helper}({name})) {{"),
            format!("  {errors_name}.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn compare(
    indent: &str,
    expression: String,
    errors_name: &str,
    literal: &str,
    message: &str,
) -> String {
    block(
        indent,
        [
            format!("if ({expression}) {{"),
            format!("  {errors_name}.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn string_contains(
    indent: &str,
    name: &str,
    errors_name: &str,
    rule: &Value,
    positive: bool,
    literal: &str,
    message: &str,
) -> String {
    let pattern = rule.get_attr("pattern").unwrap().to_string();
    let prefix = if positive { "" } else { "!" };
    block(
        indent,
        [
            format!("if ({prefix}{name}.contains({pattern})) {{"),
            format!("  {errors_name}.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn regex(
    indent: &str,
    name: &str,
    errors_name: &str,
    rule: &Value,
    literal: &str,
    message: &str,
) -> String {
    let pattern = rule.get_attr("pattern").unwrap().to_string();
    block(
        indent,
        [
            format!("if (!RegExp({pattern}).hasMatch({name})) {{"),
            format!("  {errors_name}.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn must_match(
    indent: &str,
    name: &str,
    self_name: &str,
    errors_name: &str,
    rule: &Value,
    literal: &str,
    message: &str,
) -> String {
    let other = rule.get_attr("other").unwrap().to_string();
    block(
        indent,
        [
            format!("if ({name} != {self_name}.{other}) {{"),
            format!("  {errors_name}.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn nested(
    indent: &str,
    field_name: &str,
    value_name: &str,
    errors_name: &str,
    validation_name: &str,
    nested_errors_name: &str,
    nested_error_name: &str,
) -> String {
    block(
        indent,
        [
            format!("final {validation_name} = {value_name}.validate();"),
            format!("if ({validation_name} case Invalid(errors: final {nested_errors_name})) {{"),
            format!("  for (final {nested_error_name} in {nested_errors_name}) {{"),
            format!(
                "    {errors_name}.add(ValidationError(field: '{field_name}.${{{nested_error_name}.field}}', message: {nested_error_name}.message));"
            ),
            "  }".to_owned(),
            "}".to_owned(),
        ],
    )
}

fn custom(
    indent: &str,
    name: &str,
    errors_name: &str,
    custom_error_name: &str,
    rule: &Value,
) -> String {
    let custom = rule.get_attr("custom").unwrap().to_string();
    block(
        indent,
        [
            format!("final {custom_error_name} = {custom}({name});"),
            format!("if ({custom_error_name} != null) {{"),
            format!("  {errors_name}.add({custom_error_name});"),
            "}".to_owned(),
        ],
    )
}

fn block<const N: usize>(indent: &str, lines: [String; N]) -> String {
    lines
        .into_iter()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
