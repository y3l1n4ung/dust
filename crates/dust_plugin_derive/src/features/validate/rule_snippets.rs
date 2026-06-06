use minijinja::Value;

pub(crate) fn rule_line(field: Value, rule: Value, indent: String) -> String {
    let name = field.get_attr("name").unwrap().to_string();
    let literal = field.get_attr("literal").unwrap().to_string();
    let kind = rule.get_attr("kind").unwrap().to_string();
    let message = rule.get_attr("message").unwrap().to_string();
    match kind.as_str() {
        "email" => simple_helper(&indent, &name, &literal, &message, "isEmail"),
        "url" => simple_helper(&indent, &name, &literal, &message, "isUrl"),
        "length_equal" => compare(
            &indent,
            format!("{name}.length != {}", rule.get_attr("equal").unwrap()),
            &literal,
            &message,
        ),
        "length_min" => compare(
            &indent,
            format!("{name}.length < {}", rule.get_attr("min").unwrap()),
            &literal,
            &message,
        ),
        "length_max" => compare(
            &indent,
            format!("{name}.length > {}", rule.get_attr("min").unwrap()),
            &literal,
            &message,
        ),
        "range_min" => compare(
            &indent,
            format!("{name} < {}", rule.get_attr("max").unwrap()),
            &literal,
            &message,
        ),
        "range_max" => compare(
            &indent,
            format!("{name} > {}", rule.get_attr("max").unwrap()),
            &literal,
            &message,
        ),
        "contains" => string_contains(&indent, &name, &rule, false, &literal, &message),
        "does_not_contain" => string_contains(&indent, &name, &rule, true, &literal, &message),
        "regex" => regex(&indent, &name, &rule, &literal, &message),
        "must_match" => must_match(&indent, &name, &rule, &literal, &message),
        "nested" => nested(&indent, &name),
        "custom" => custom(&indent, &name, &rule),
        _ => String::new(),
    }
}

fn simple_helper(indent: &str, name: &str, literal: &str, message: &str, helper: &str) -> String {
    block(
        indent,
        [
            format!("if (!ValidationHelper.{helper}({name})) {{"),
            format!("  errors.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn compare(indent: &str, expression: String, literal: &str, message: &str) -> String {
    block(
        indent,
        [
            format!("if ({expression}) {{"),
            format!("  errors.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn string_contains(
    indent: &str,
    name: &str,
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
            format!("  errors.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn regex(indent: &str, name: &str, rule: &Value, literal: &str, message: &str) -> String {
    let pattern = rule.get_attr("pattern").unwrap().to_string();
    block(
        indent,
        [
            format!("if (!RegExp({pattern}).hasMatch({name})) {{"),
            format!("  errors.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn must_match(indent: &str, name: &str, rule: &Value, literal: &str, message: &str) -> String {
    let other = rule.get_attr("other").unwrap().to_string();
    block(
        indent,
        [
            format!("if ({name} != self.{other}) {{"),
            format!("  errors.add(ValidationError(field: {literal}, message: {message}));"),
            "}".to_owned(),
        ],
    )
}

fn nested(indent: &str, name: &str) -> String {
    block(
        indent,
        [
            format!("final {name}Validation = {name}.validate();"),
            format!("if ({name}Validation case Invalid(errors: final nestedErrors)) {{"),
            "  for (final error in nestedErrors) {".to_owned(),
            format!(
                "    errors.add(ValidationError(field: '{name}.${{error.field}}', message: error.message));"
            ),
            "  }".to_owned(),
            "}".to_owned(),
        ],
    )
}

fn custom(indent: &str, name: &str, rule: &Value) -> String {
    let custom = rule.get_attr("custom").unwrap().to_string();
    block(
        indent,
        [
            format!("final {name}CustomError = {custom}({name});"),
            format!("if ({name}CustomError != null) {{"),
            format!("  errors.add({name}CustomError);"),
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
