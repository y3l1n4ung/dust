use dust_diagnostics::Diagnostic;
use dust_ir::SerdeRenameRuleIr;

pub(crate) fn parse_serde_arguments<'a>(
    source: Option<&'a str>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(&'a str, &'a str)> {
    let Some(source) = source.map(str::trim).filter(|source| !source.is_empty()) else {
        return Vec::new();
    };

    let Some(inner) = source
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
    else {
        diagnostics.push(Diagnostic::error(
            "SerDe config arguments must use parenthesized named arguments",
        ));
        return Vec::new();
    };

    let inner = inner.trim();
    if inner.is_empty() {
        return Vec::new();
    }

    let mut arguments = Vec::new();
    for item in super::type_parse::split_top_level_items(inner) {
        if let Some((key, value)) = split_named_argument(item) {
            arguments.push((key.trim(), value.trim()));
        } else {
            diagnostics.push(Diagnostic::error(format!(
                "could not parse `SerDe` argument `{item}`"
            )));
        }
    }

    arguments
}

pub(crate) fn parse_string_literal(source: &str) -> Option<String> {
    let source = source.trim();
    let first = source.chars().next()?;
    let last = source.chars().next_back()?;
    if source.len() < 2 || first != last || !matches!(first, '\'' | '"') {
        return None;
    }

    Some(source[1..source.len() - 1].to_owned())
}

pub(crate) fn parse_bool_literal(source: &str) -> Option<bool> {
    match source.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub(crate) fn parse_string_list(source: &str) -> Option<Vec<String>> {
    let source = source.trim();
    let inner = source.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }

    super::type_parse::split_top_level_items(inner)
        .into_iter()
        .map(parse_string_literal)
        .collect()
}

pub(crate) fn parse_serde_rename_rule(source: &str) -> Option<SerdeRenameRuleIr> {
    match source.trim().rsplit('.').next()? {
        "lowerCase" => Some(SerdeRenameRuleIr::LowerCase),
        "upperCase" => Some(SerdeRenameRuleIr::UpperCase),
        "pascalCase" => Some(SerdeRenameRuleIr::PascalCase),
        "camelCase" => Some(SerdeRenameRuleIr::CamelCase),
        "snakeCase" => Some(SerdeRenameRuleIr::SnakeCase),
        "screamingSnakeCase" => Some(SerdeRenameRuleIr::ScreamingSnakeCase),
        "kebabCase" => Some(SerdeRenameRuleIr::KebabCase),
        "screamingKebabCase" => Some(SerdeRenameRuleIr::ScreamingKebabCase),
        _ => None,
    }
}

fn split_named_argument(source: &str) -> Option<(&str, &str)> {
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;
    let mut quote = None;
    let mut escape = false;

    for (index, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            ':' if depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0 =>
            {
                return Some((&source[..index], &source[index + 1..]));
            }
            _ => {}
        }
    }

    None
}
