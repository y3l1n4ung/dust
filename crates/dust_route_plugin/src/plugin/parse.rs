use dust_ir::{ConfigApplicationIr, SymbolId};
use dust_parser_dart::ParsedAnnotation;

#[cfg(test)]
use dust_ir::SpanIr;
#[cfg(test)]
use dust_text::{FileId, TextRange};

use super::{
    constants::ROUTE,
    model::{RouteAnnotation, RouterAnnotation},
};

/// Returns the `@AppRoute` config from a lowered class, if present.
pub(crate) fn route_config(configs: &[ConfigApplicationIr]) -> Option<&ConfigApplicationIr> {
    configs
        .iter()
        .find(|config| config_name(&config.symbol) == ROUTE)
}

#[cfg(test)]
/// Parses route annotation arguments in parser unit tests.
pub(crate) fn parse_route_annotation(args: Option<&str>) -> Option<RouteAnnotation> {
    parse_route_config(&test_config(ROUTE, args))
}

/// Parses a lowered `@AppRoute` annotation.
pub(crate) fn parse_route_config(config: &ConfigApplicationIr) -> Option<RouteAnnotation> {
    let path = config.positional_string(0)?;
    let name = config.named_string("name");
    let shell = config.named_type("shell");
    let guards_configured = config.has_named_argument("guards");
    let guards = config.named_type_list("guards").unwrap_or_default();
    let transition = config
        .named_expression_source("transition")
        .map(normalize_transition_source);
    let fullscreen_dialog = config.named_bool("fullscreenDialog").unwrap_or(false);
    let maintain_state = config.named_bool("maintainState").unwrap_or(true);
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

/// Parses a source-surface `@AppRoute` annotation for workspace analysis.
pub(crate) fn parse_route_surface(annotation: &ParsedAnnotation) -> Option<RouteAnnotation> {
    let path = annotation.positional_string(0)?;
    let name = annotation.named_string("name");
    let shell = annotation.named_type("shell");
    let guards_configured = annotation.has_named_argument("guards");
    let guards = annotation.named_type_list("guards").unwrap_or_default();
    let transition = annotation
        .named_expression_source("transition")
        .map(normalize_transition_source);
    let fullscreen_dialog = annotation.named_bool("fullscreenDialog").unwrap_or(false);
    let maintain_state = annotation.named_bool("maintainState").unwrap_or(true);
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
/// Parses router annotation arguments in parser unit tests.
pub(crate) fn parse_router_annotation(args: Option<&str>) -> RouterAnnotation {
    parse_router_config(Some(&test_config("AppRouter", args)))
}

/// Parses a lowered `@AppRouter` annotation.
pub(crate) fn parse_router_config(config: Option<&ConfigApplicationIr>) -> RouterAnnotation {
    let Some(config) = config else {
        return RouterAnnotation {
            initial: None,
            not_found: None,
        };
    };

    RouterAnnotation {
        initial: config.named_string("initial"),
        not_found: config.named_string("notFound"),
    }
}

/// Parses a source-surface `@AppRouter` annotation for workspace analysis.
pub(crate) fn parse_router_surface(annotation: &ParsedAnnotation) -> RouterAnnotation {
    RouterAnnotation {
        initial: annotation.named_string("initial"),
        not_found: annotation.named_string("notFound"),
    }
}

/// Returns the simple name for a resolved annotation symbol.
fn config_name(symbol: &SymbolId) -> &str {
    symbol.0.rsplit("::").next().unwrap_or(symbol.0.as_str())
}

/// Normalizes transition builder source captured from annotation arguments.
fn normalize_transition_source(source: String) -> String {
    let source = source
        .trim()
        .strip_prefix("const ")
        .unwrap_or(source.trim());
    for prefix in ["cupertino.", "material."] {
        if let Some(stripped) = source.strip_prefix(prefix) {
            return stripped.to_owned();
        }
    }
    source.to_owned()
}

#[cfg(test)]
/// Builds a lowered config for parser unit tests.
fn test_config(name: &str, args: Option<&str>) -> ConfigApplicationIr {
    ConfigApplicationIr::new(
        SymbolId::new(name),
        args.map(str::to_owned),
        SpanIr::new(FileId::default(), TextRange::default()),
    )
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
        let router = parse_router_annotation(Some("(initial: '/', notFound: '/404')"));
        assert_eq!(router.initial.as_deref(), Some("/"));
        assert_eq!(router.not_found.as_deref(), Some("/404"));
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

    #[test]
    fn normalizes_known_flutter_transition_prefixes() {
        let route = parse_route_annotation(Some(
            "('/login', transition: const cupertino.CupertinoPageTransitionsBuilder())",
        ))
        .unwrap();
        assert_eq!(
            route.transition.as_deref(),
            Some("CupertinoPageTransitionsBuilder()")
        );
    }
}
