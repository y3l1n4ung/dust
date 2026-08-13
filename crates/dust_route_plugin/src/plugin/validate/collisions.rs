use dust_diagnostics::Diagnostic;

/// Route details used to explain route collisions.
#[derive(Clone)]
pub(super) struct RouteCollision {
    /// Page class annotated as a route.
    pub(super) page_class: String,
    /// Absolute route URL pattern.
    pub(super) path: String,
    /// Optional explicit route name.
    pub(super) name: Option<String>,
}

/// Builds a duplicate route path diagnostic with both conflicting pages.
pub(super) fn duplicate_route_path_diagnostic(
    path: &str,
    first: &RouteCollision,
    second: &RouteCollision,
) -> Diagnostic {
    Diagnostic::error(format!(
        "duplicate route path `{path}` used by {} and {}; URL `{}` matches both",
        route_with_name(first),
        route_with_name(second),
        concrete_route_url(path)
    ))
}

/// Builds a duplicate route name diagnostic with both conflicting pages.
pub(super) fn duplicate_route_name_diagnostic(
    name: &str,
    first: &RouteCollision,
    second: &RouteCollision,
) -> Diagnostic {
    Diagnostic::error(format!(
        "duplicate route name `{name}` used by {} and {}",
        route_with_path(first),
        route_with_path(second)
    ))
}

/// Renders a page and optional route name for collision diagnostics.
fn route_with_name(route: &RouteCollision) -> String {
    match route.name.as_deref() {
        Some(name) => format!("`{}` (`{name}`)", route.page_class),
        None => format!("`{}`", route.page_class),
    }
}

/// Renders a page and route path for duplicate name diagnostics.
fn route_with_path(route: &RouteCollision) -> String {
    format!("`{}` (`{}`)", route.page_class, route.path)
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
