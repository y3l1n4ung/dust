use dust_dart_emit::{parse_bool_literal, parse_string_literal, split_top_level_items};
use dust_ir::{ConfigApplicationIr, SymbolId};
use dust_parser_dart::ParsedAnnotation;

#[cfg(test)]
use dust_dart_emit::{normalized_args, parse_named_arguments, split_top_level_once};

use super::{
    constants::ROUTE,
    model::{RouteAnnotation, RouterAnnotation},
};

pub(crate) fn route_config(configs: &[ConfigApplicationIr]) -> Option<&ConfigApplicationIr> {
    configs
        .iter()
        .find(|config| config_name(&config.symbol) == ROUTE)
}

#[cfg(test)]
pub(crate) fn parse_route_annotation(args: Option<&str>) -> Option<RouteAnnotation> {
    let args = args?;
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

pub(crate) fn parse_route_config(config: &ConfigApplicationIr) -> Option<RouteAnnotation> {
    let path = config
        .positional_argument_source(0)
        .and_then(parse_string_literal)?;
    let name = config
        .named_argument_source("name")
        .and_then(parse_string_literal);
    let shell = config
        .named_argument_source("shell")
        .and_then(parse_type_name);
    let guards_configured = config.has_named_argument("guards");
    let guards = config
        .named_argument_source("guards")
        .map(type_list)
        .unwrap_or_default();
    let transition = config
        .named_argument_source("transition")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let fullscreen_dialog = config
        .named_argument_source("fullscreenDialog")
        .and_then(parse_bool_literal)
        .unwrap_or(false);
    let maintain_state = config
        .named_argument_source("maintainState")
        .and_then(parse_bool_literal)
        .unwrap_or(true);
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

pub(crate) fn parse_route_surface(annotation: &ParsedAnnotation) -> Option<RouteAnnotation> {
    let path = annotation
        .positional_argument_source(0)
        .and_then(parse_string_literal)?;
    let name = annotation
        .named_argument_source("name")
        .and_then(parse_string_literal);
    let shell = annotation
        .named_argument_source("shell")
        .and_then(parse_type_name);
    let guards_configured = annotation.has_named_argument("guards");
    let guards = annotation
        .named_argument_source("guards")
        .map(type_list)
        .unwrap_or_default();
    let transition = annotation
        .named_argument_source("transition")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let fullscreen_dialog = annotation
        .named_argument_source("fullscreenDialog")
        .and_then(parse_bool_literal)
        .unwrap_or(false);
    let maintain_state = annotation
        .named_argument_source("maintainState")
        .and_then(parse_bool_literal)
        .unwrap_or(true);
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

#[cfg(test)]
pub(crate) fn parse_router_annotation(args: Option<&str>) -> RouterAnnotation {
    let Some(args) = args else {
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

pub(crate) fn parse_router_config(config: Option<&ConfigApplicationIr>) -> RouterAnnotation {
    let Some(config) = config else {
        return RouterAnnotation {
            initial: None,
            not_found: None,
        };
    };

    RouterAnnotation {
        initial: config
            .named_argument_source("initial")
            .and_then(parse_type_name),
        not_found: config
            .named_argument_source("notFound")
            .and_then(parse_type_name),
    }
}

pub(crate) fn parse_router_surface(annotation: &ParsedAnnotation) -> RouterAnnotation {
    RouterAnnotation {
        initial: annotation
            .named_argument_source("initial")
            .and_then(parse_type_name),
        not_found: annotation
            .named_argument_source("notFound")
            .and_then(parse_type_name),
    }
}

fn config_name(symbol: &SymbolId) -> &str {
    symbol.0.rsplit("::").next().unwrap_or(symbol.0.as_str())
}

#[cfg(test)]
fn first_string_literal(args: &str) -> Option<String> {
    let first = split_top_level_items(normalized_args(args)?)
        .first()
        .copied()?;
    if split_top_level_once(first, ':').is_some() {
        return None;
    }
    parse_string_literal(first)
}

#[cfg(test)]
fn named_string_literal(args: &str, name: &str) -> Option<String> {
    parse_string_literal(named_value_source(args, name)?)
}

#[cfg(test)]
fn named_type_literal(args: &str, name: &str) -> Option<String> {
    let value = named_value_source(args, name)?.trim();
    parse_type_name(value)
}

#[cfg(test)]
fn named_member_literal(args: &str, name: &str) -> Option<String> {
    let value = named_value_source(args, name)?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
fn named_bool_literal(args: &str, name: &str) -> Option<bool> {
    parse_bool_literal(named_value_source(args, name)?)
}

#[cfg(test)]
fn named_type_list(args: &str, name: &str) -> Vec<String> {
    let Some(value) = named_value_source(args, name).map(str::trim) else {
        return Vec::new();
    };
    type_list(value)
}

fn type_list(value: &str) -> Vec<String> {
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    split_top_level_items(inner)
        .into_iter()
        .filter_map(parse_type_name)
        .collect()
}

fn parse_type_name(value: &str) -> Option<String> {
    let ident = value
        .trim()
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '.')
        .collect::<String>();
    (!ident.is_empty()).then_some(ident)
}

#[cfg(test)]
fn named_value_source<'a>(args: &'a str, name: &str) -> Option<&'a str> {
    for (key, value) in parse_named_arguments(Some(args)) {
        if key.trim() == name {
            return Some(value.trim());
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
