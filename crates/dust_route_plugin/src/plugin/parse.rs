use dust_ir::{ConfigApplicationIr, SymbolId};

use super::{
    constants::ROUTE,
    model::{RouteAnnotation, RouterAnnotation},
};

pub(crate) fn route_config(configs: &[ConfigApplicationIr]) -> Option<&ConfigApplicationIr> {
    configs
        .iter()
        .find(|config| config_name(&config.symbol) == ROUTE)
}

pub(crate) fn parse_route_annotation(args: Option<&str>) -> Option<RouteAnnotation> {
    let args = normalize_args(args?)?;
    let path = first_string_literal(args)?;
    let name = named_string_literal(args, "name");
    let shell = named_type_literal(args, "shell");
    let guards_configured = named_value_source(args, "guards").is_some();
    let guards = named_type_list(args, "guards");
    let transition = named_member_literal(args, "transition");
    let fullscreen_dialog = named_bool_literal(args, "fullscreenDialog").unwrap_or(false);
    let maintain_state = named_bool_literal(args, "maintainState").unwrap_or(true);
    Some(RouteAnnotation {
        path,
        name,
        shell,
        guards,
        guards_configured,
        transition,
        fullscreen_dialog,
        maintain_state,
    })
}

pub(crate) fn parse_router_annotation(args: Option<&str>) -> RouterAnnotation {
    let Some(args) = args.and_then(normalize_args) else {
        return RouterAnnotation {
            initial: None,
            not_found: None,
        };
    };

    RouterAnnotation {
        initial: named_type_literal(args, "initial"),
        not_found: named_type_literal(args, "notFound"),
    }
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

fn first_string_literal(args: &str) -> Option<String> {
    let args = args.trim_start();
    parse_quoted_string(args).map(|(value, _)| value)
}

fn named_string_literal(args: &str, name: &str) -> Option<String> {
    let value = named_value_source(args, name)?;
    parse_quoted_string(value.trim()).map(|(value, _)| value)
}

fn named_type_literal(args: &str, name: &str) -> Option<String> {
    let value = named_value_source(args, name)?.trim();
    parse_type_name(value)
}

fn named_member_literal(args: &str, name: &str) -> Option<String> {
    let value = named_value_source(args, name)?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn named_bool_literal(args: &str, name: &str) -> Option<bool> {
    match named_value_source(args, name)?.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn named_type_list(args: &str, name: &str) -> Vec<String> {
    let Some(value) = named_value_source(args, name).map(str::trim) else {
        return Vec::new();
    };
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    inner.split(',').filter_map(parse_type_name).collect()
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
    let mut bracket_depth = 0_usize;
    let mut paren_depth = 0_usize;
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
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            ',' if bracket_depth == 0 && paren_depth == 0 => {
                args.push(source[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    args.push(source[start..].trim());
    args
}

fn parse_quoted_string(source: &str) -> Option<(String, &str)> {
    let mut chars = source.char_indices();
    let (_, quote) = chars.next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let mut escaped = false;
    let mut value = String::new();
    for (idx, ch) in chars {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            found if found == quote => return Some((value, &source[idx + ch.len_utf8()..])),
            other => value.push(other),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{parse_route_annotation, parse_router_annotation};

    #[test]
    fn parses_route_metadata() {
        let route = parse_route_annotation(Some(
            "('/projects/:id', name: 'project', shell: AppShell, guards: [AuthGuard], transition: FadeUpwardsPageTransitionsBuilder(), fullscreenDialog: true)",
        ))
        .unwrap();
        assert_eq!(route.path, "/projects/:id");
        assert_eq!(route.name.as_deref(), Some("project"));
        assert_eq!(route.shell.as_deref(), Some("AppShell"));
        assert_eq!(route.guards, vec!["AuthGuard".to_string()]);
        assert!(route.guards_configured);
        assert_eq!(
            route.transition.as_deref(),
            Some("FadeUpwardsPageTransitionsBuilder()")
        );
        assert!(route.fullscreen_dialog);
        assert!(route.maintain_state);
    }

    #[test]
    fn parses_router_initial_and_not_found() {
        let router =
            parse_router_annotation(Some("(initial: DashboardPage, notFound: NotFoundPage)"));
        assert_eq!(router.initial.as_deref(), Some("DashboardPage"));
        assert_eq!(router.not_found.as_deref(), Some("NotFoundPage"));
    }

    #[test]
    fn parses_named_values_only_at_top_level() {
        let route = parse_route_annotation(Some(
            r#"('/search?name:ignored', name: 'search', transition: SharedAxisPageTransitionsBuilder(label: 'name:still ignored'), guards: [AuthGuard, BillingGuard])"#,
        ))
        .unwrap();
        assert_eq!(route.path, "/search?name:ignored");
        assert_eq!(route.name.as_deref(), Some("search"));
        assert_eq!(
            route.transition.as_deref(),
            Some("SharedAxisPageTransitionsBuilder(label: 'name:still ignored')")
        );
        assert_eq!(
            route.guards,
            vec!["AuthGuard".to_string(), "BillingGuard".to_string()]
        );
    }
}
