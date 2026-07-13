use dust_ir::{ConfigApplicationIr, NormalizedConfigIr, RouteConfigIr, RouterConfigIr};
use dust_parser_dart::{
    ParsedAnnotation, ParsedAnnotationNamedArgument, ParsedAnnotationValue,
    ParsedAnnotationValueRootKind,
};

use super::model::{RouteAnnotation, RouterAnnotation};

/// Returns resolver-normalized `AppRoute` configuration.
pub(crate) fn route_config(configs: &[ConfigApplicationIr]) -> Option<&RouteConfigIr> {
    configs.iter().find_map(|config| match &config.normalized {
        Some(NormalizedConfigIr::Route(route)) => Some(route),
        _ => None,
    })
}

/// Copies resolver-normalized route configuration into a plugin route fact.
pub(crate) fn route_annotation(config: &RouteConfigIr) -> RouteAnnotation {
    RouteAnnotation {
        path: config.path.clone(),
        name: config.name.clone(),
        shell: config.shell.clone(),
        guards: config.guards.clone(),
        guards_configured: config.guards_configured,
        transition: config.transition.clone(),
        fullscreen_dialog: config.fullscreen_dialog,
        maintain_state: config.maintain_state,
    }
}

/// Returns resolver-normalized `AppRouter` configuration.
pub(crate) fn router_config(configs: &[ConfigApplicationIr]) -> Option<&RouterConfigIr> {
    configs.iter().find_map(|config| match &config.normalized {
        Some(NormalizedConfigIr::Router(router)) => Some(router),
        _ => None,
    })
}

/// Parses a source-surface `@AppRoute` annotation for workspace analysis.
pub(crate) fn parse_route_surface(annotation: &ParsedAnnotation) -> Option<RouteAnnotation> {
    let arguments = annotation.parsed_arguments.as_ref()?;
    let path = parsed_string(arguments.positional.first()?.value.as_ref()?)?.to_owned();
    let named = |name| {
        arguments
            .named
            .iter()
            .find(|argument| argument.name == name)
    };
    let name = named("name")
        .and_then(parsed_value)
        .and_then(parsed_string)
        .map(str::to_owned);
    let shell = named("shell")
        .and_then(parsed_value)
        .and_then(parsed_member)
        .map(str::to_owned);
    let guards_configured = named("guards").is_some();
    let guards = named("guards")
        .and_then(parsed_value)
        .map(parsed_member_list)
        .unwrap_or_default();
    let transition = named("transition")
        .map(|argument| normalize_transition_source(argument.value_source.clone()));
    let fullscreen_dialog = named("fullscreenDialog")
        .and_then(parsed_value)
        .and_then(parsed_bool)
        .unwrap_or(false);
    let maintain_state = named("maintainState")
        .and_then(parsed_value)
        .and_then(parsed_bool)
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

/// Returns the parser-owned value for one named argument.
fn parsed_value(argument: &ParsedAnnotationNamedArgument) -> Option<&ParsedAnnotationValue> {
    argument.value.as_ref()
}

/// Returns a parser-owned string literal value.
fn parsed_string(value: &ParsedAnnotationValue) -> Option<&str> {
    match &value.kind {
        ParsedAnnotationValueRootKind::String(value) => Some(value),
        _ => None,
    }
}

/// Returns a parser-owned member reference.
fn parsed_member(value: &ParsedAnnotationValue) -> Option<&str> {
    match &value.kind {
        ParsedAnnotationValueRootKind::Member(value) => Some(value),
        _ => None,
    }
}

/// Returns member references from a parser-owned list literal.
fn parsed_member_list(value: &ParsedAnnotationValue) -> Vec<String> {
    match &value.kind {
        ParsedAnnotationValueRootKind::List(values) => values
            .iter()
            .filter_map(parsed_member)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

/// Returns a parser-owned boolean literal value.
fn parsed_bool(value: &ParsedAnnotationValue) -> Option<bool> {
    match value.kind {
        ParsedAnnotationValueRootKind::Bool(value) => Some(value),
        _ => None,
    }
}

/// Parses a source-surface `@AppRouter` annotation for workspace analysis.
pub(crate) fn parse_router_surface(annotation: &ParsedAnnotation) -> RouterAnnotation {
    RouterAnnotation {
        initial: annotation.named_string("initial"),
        not_found: annotation.named_string("notFound"),
    }
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
