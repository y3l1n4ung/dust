use dust_ir::{ConfigApplicationIr, SymbolId};

use super::{constants::VIEW_MODEL, model::ViewModelAnnotation};

pub(crate) fn view_model_config(configs: &[ConfigApplicationIr]) -> Option<&ConfigApplicationIr> {
    configs
        .iter()
        .find(|config| config_name(&config.symbol) == VIEW_MODEL)
}

pub(crate) fn parse_view_model_annotation(args: Option<&str>) -> Option<ViewModelAnnotation> {
    let args = normalize_args(args?)?;
    let state_type = named_type_literal(args, "state").or_else(|| first_type_literal(args))?;
    let args_type = named_type_literal(args, "args");
    let initial_source = named_value_source(args, "initial").map(str::to_owned);
    Some(ViewModelAnnotation {
        state_type,
        args_type,
        initial_source,
    })
}

fn config_name(symbol: &SymbolId) -> &str {
    symbol.0.rsplit("::").next().unwrap_or(symbol.0.as_str())
}

fn normalize_args(args: &str) -> Option<&str> {
    let trimmed = args.trim();
    trimmed
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .map(str::trim)
}

fn first_type_literal(args: &str) -> Option<String> {
    let first = top_level_args(args).first().copied()?;
    if first.contains(':') {
        return None;
    }
    parse_type_name(first)
}

fn named_type_literal(args: &str, name: &str) -> Option<String> {
    let value = named_value_source(args, name)?.trim();
    parse_type_name(value)
}

fn parse_type_name(value: &str) -> Option<String> {
    let ident = value
        .trim()
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '.')
        .collect::<String>();
    (!ident.is_empty()).then_some(ident)
}

fn named_value_source<'a>(args: &'a str, name: &str) -> Option<&'a str> {
    for arg in top_level_args(args) {
        let Some((key, value)) = arg.split_once(':') else {
            continue;
        };
        if key.trim() == name {
            return Some(value.trim());
        }
    }
    None
}

fn top_level_args(source: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0_usize;
    let mut bracket_depth = 0_usize;
    let mut quote = None;
    let mut escaped = false;

    for (index, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                args.push(source[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    args.push(source[start..].trim());
    args
}

#[cfg(test)]
mod tests {
    use super::parse_view_model_annotation;

    #[test]
    fn parses_named_state_and_args() {
        let annotation =
            parse_view_model_annotation(Some("(state: TaskBoardState, args: TaskBoardArgs)"))
                .unwrap();
        assert_eq!(annotation.state_type, "TaskBoardState");
        assert_eq!(annotation.args_type.as_deref(), Some("TaskBoardArgs"));
        assert_eq!(annotation.initial_source, None);
    }
}

#[cfg(test)]
mod initial_tests {
    use super::parse_view_model_annotation;

    #[test]
    fn parses_initial_expression_source() {
        let annotation = parse_view_model_annotation(Some(
            "(state: ShellTab, args: ShellViewModelArgs, initial: ShellTab.dashboard)",
        ))
        .unwrap();
        assert_eq!(
            annotation.initial_source.as_deref(),
            Some("ShellTab.dashboard")
        );
    }
}
