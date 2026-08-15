use std::collections::{HashMap, HashSet};

use dust_diagnostics::Diagnostic;

use crate::plugin::model::RouteSpec;

use super::identifiers::{is_dart_reserved_word, is_valid_dart_identifier};

/// Generated declarations that share Dart's library namespace with route classes.
pub(super) struct GeneratedRouteNames<'a> {
    /// Generated base class extended by the handwritten router.
    pub(super) generated_base_class: &'a str,
    /// Generated sealed route base class.
    pub(super) route_base_class: &'a str,
    /// Generated BuildContext extension.
    pub(super) context_extension: &'a str,
    /// Generated typed navigator facade class.
    pub(super) navigator_class: &'a str,
    /// Generated typed route action class.
    pub(super) route_action_class: &'a str,
}

/// Validates duplicate route paths, names, helpers, classes, and ambiguous URLs.
pub(super) fn validate_workspace_route_set(
    routes: &[RouteSpec],
    generated_names: &GeneratedRouteNames<'_>,
    workspace_classes: &HashSet<String>,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut paths = HashMap::new();
    let mut names = HashMap::new();
    let mut route_classes = HashSet::new();
    let mut helper_names = HashSet::new();
    let generated_support_declarations = HashSet::from([
        generated_names.generated_base_class,
        generated_names.route_base_class,
        generated_names.context_extension,
        generated_names.navigator_class,
        generated_names.route_action_class,
    ]);
    for route in routes {
        if let Some(previous) = paths.insert(route.path.clone(), route) {
            diagnostics.push(duplicate_route_path_diagnostic(
                &route.path,
                previous,
                route,
            ));
        }
        if let Some(previous) = names.insert(route.name.clone(), route) {
            diagnostics.push(duplicate_route_name_diagnostic(
                &route.name,
                previous,
                route,
            ));
        }
        if !is_valid_dart_identifier(&route.name) || is_dart_reserved_word(&route.name) {
            diagnostics.push(Diagnostic::error(format!(
                "route name `{}` must be a valid non-reserved Dart identifier",
                route.name
            )));
        }
        if route.name == "pop" {
            diagnostics.push(Diagnostic::error(
                "route name `pop` conflicts with the generated navigator `pop` helper",
            ));
        }
        if !helper_names.insert(route.name.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "generated route helper `{}` is emitted more than once",
                route.name
            )));
        }
        if !route_classes.insert(route.route_class.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "generated route class `{}` is emitted by more than one route name",
                route.route_class
            )));
        }
        if generated_support_declarations.contains(route.route_class.as_str()) {
            diagnostics.push(Diagnostic::error(format!(
                "generated route class `{}` conflicts with a generated router support declaration",
                route.route_class
            )));
        }
        if workspace_classes.contains(&route.route_class) {
            diagnostics.push(Diagnostic::error(format!(
                "generated route class `{}` conflicts with an existing Dart class; rename the route or page",
                route.route_class
            )));
        }
        validate_duplicate_path_params(route, &mut diagnostics);
    }
    validate_ambiguous_path_siblings(routes, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

/// Builds a duplicate route path diagnostic with both conflicting pages.
fn duplicate_route_path_diagnostic(
    path: &str,
    first: &RouteSpec,
    second: &RouteSpec,
) -> Diagnostic {
    Diagnostic::error(format!(
        "duplicate route path `{path}` used by {} and {}; URL `{}` matches both",
        route_with_name(first),
        route_with_name(second),
        concrete_route_url(path)
    ))
}

/// Builds a duplicate route name diagnostic with both conflicting pages.
fn duplicate_route_name_diagnostic(
    name: &str,
    first: &RouteSpec,
    second: &RouteSpec,
) -> Diagnostic {
    Diagnostic::error(format!(
        "duplicate route name `{name}` used by {} and {}",
        route_with_path(first),
        route_with_path(second)
    ))
}

/// Renders a route page with its generated action name.
fn route_with_name(route: &RouteSpec) -> String {
    format!("`{}` (`{}`)", route.page_class, route.name)
}

/// Renders a route page with its URL pattern.
fn route_with_path(route: &RouteSpec) -> String {
    format!("`{}` (`{}`)", route.page_class, route.path)
}

/// Rejects paths that bind the same `:param` name more than once.
fn validate_duplicate_path_params(route: &RouteSpec, diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = HashSet::new();
    let mut reported = HashSet::new();
    for segment in path_segments(&route.path) {
        let Some(param) = path_param_name(segment) else {
            continue;
        };
        if !seen.insert(param) && reported.insert(param) {
            diagnostics.push(Diagnostic::error(format!(
                "route `{}` path `{}` declares duplicate path parameter `:{param}`",
                route.page_class, route.path
            )));
        }
    }
}

/// Rejects same-length path patterns with overlapping static/dynamic segments.
fn validate_ambiguous_path_siblings(routes: &[RouteSpec], diagnostics: &mut Vec<Diagnostic>) {
    for (index, route) in routes.iter().enumerate() {
        let route_segments = path_segments(&route.path).collect::<Vec<_>>();
        for sibling in routes.iter().skip(index + 1) {
            let sibling_segments = path_segments(&sibling.path).collect::<Vec<_>>();
            let Some(parent) = ambiguous_parent(&route_segments, &sibling_segments) else {
                continue;
            };
            diagnostics.push(ambiguous_sibling_diagnostic(
                sibling,
                route,
                &parent,
                &ambiguous_example_url(&route_segments, &sibling_segments),
            ));
        }
    }
}

/// Returns the shared parent path when sibling patterns can match the same URL.
fn ambiguous_parent(left: &[&str], right: &[&str]) -> Option<String> {
    if left.len() != right.len() {
        return None;
    }

    let mut first_static_dynamic_parent = None;
    for (index, (left_segment, right_segment)) in left.iter().zip(right).enumerate() {
        if left_segment == right_segment {
            continue;
        }
        let left_dynamic = path_param_name(left_segment).is_some();
        let right_dynamic = path_param_name(right_segment).is_some();
        if !left_dynamic && !right_dynamic {
            return None;
        }
        if left_dynamic != right_dynamic {
            first_static_dynamic_parent.get_or_insert_with(|| {
                display_path(
                    &left[..index]
                        .iter()
                        .map(|segment| (*segment).to_owned())
                        .collect::<Vec<_>>(),
                )
            });
        }
    }

    first_static_dynamic_parent
}

/// Builds a diagnostic for ambiguous static and dynamic sibling segments.
fn ambiguous_sibling_diagnostic(
    route: &RouteSpec,
    sibling: &RouteSpec,
    parent: &str,
    example_url: &str,
) -> Diagnostic {
    Diagnostic::error(format!(
        "route {} conflicts with {}; URL `{example_url}` can match both because static and dynamic segments under `{parent}` are ambiguous",
        route_with_path_and_name(route),
        route_with_path_and_name(sibling)
    ))
}

/// Renders a route page with its route name and URL pattern.
fn route_with_path_and_name(route: &RouteSpec) -> String {
    format!(
        "`{}` (`{}`, `{}`)",
        route.page_class, route.name, route.path
    )
}

/// Builds one concrete URL that both ambiguous path patterns can match.
fn ambiguous_example_url(left: &[&str], right: &[&str]) -> String {
    let segments = left
        .iter()
        .zip(right)
        .enumerate()
        .map(|(index, (left_segment, right_segment))| {
            concrete_segment(index, left_segment, right_segment)
        })
        .collect::<Vec<_>>();
    display_path(&segments)
}

/// Returns a concrete URL segment shared by two ambiguous path segments.
fn concrete_segment(index: usize, left: &str, right: &str) -> String {
    match (path_param_name(left), path_param_name(right)) {
        (None, _) => left.to_owned(),
        (_, None) => right.to_owned(),
        (Some(_), Some(_)) => format!("value{}", index + 1),
    }
}

/// Builds a concrete URL by replacing dynamic segments with stable sample values.
fn concrete_route_url(path: &str) -> String {
    let segments = path_segments(path)
        .enumerate()
        .map(|(index, segment)| {
            path_param_name(segment)
                .map(|_| format!("value{}", index + 1))
                .unwrap_or_else(|| segment.to_owned())
        })
        .collect::<Vec<_>>();
    display_path(&segments)
}

/// Iterates over non-empty slash-delimited path segments.
fn path_segments(path: &str) -> impl Iterator<Item = &str> {
    path.split('/').filter(|segment| !segment.is_empty())
}

/// Returns the parameter name for a dynamic path segment.
fn path_param_name(segment: &str) -> Option<&str> {
    segment.strip_prefix(':').filter(|name| !name.is_empty())
}

/// Renders path segments with a leading slash.
fn display_path(segments: &[String]) -> String {
    if segments.is_empty() {
        "/".to_owned()
    } else {
        format!("/{}", segments.join("/"))
    }
}
