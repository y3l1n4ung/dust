use crate::plugin::model::RouteSpec;

use super::path::route_segments;

/// Returns the explicitly configured or inherited shell for a route.
pub(super) fn effective_shell<'a>(
    route: &'a RouteSpec,
    routes: &'a [RouteSpec],
) -> Option<&'a str> {
    route
        .annotation
        .shell
        .as_deref()
        .or_else(|| inherited_shell(route, routes))
}

/// Returns the explicitly configured or inherited branch for a route.
pub(super) fn effective_branch<'a>(
    route: &'a RouteSpec,
    routes: &'a [RouteSpec],
) -> Option<&'a str> {
    route
        .annotation
        .branch
        .as_deref()
        .or_else(|| inherited_branch(route, routes))
}

/// Finds the nearest parent route that declares a shell.
fn inherited_shell<'a>(route: &RouteSpec, routes: &'a [RouteSpec]) -> Option<&'a str> {
    let current_segments = route_segments(&route.path);
    routes
        .iter()
        .filter(|candidate| candidate.path != route.path)
        .filter_map(|candidate| {
            let shell = candidate.annotation.shell.as_deref()?;
            let candidate_segments = route_segments(&candidate.path);
            (candidate_segments.len() < current_segments.len()
                && current_segments.starts_with(&candidate_segments))
            .then_some((candidate_segments.len(), shell))
        })
        .max_by_key(|(length, _)| *length)
        .map(|(_, shell)| shell)
}

/// Finds the nearest parent route that declares a branch.
fn inherited_branch<'a>(route: &RouteSpec, routes: &'a [RouteSpec]) -> Option<&'a str> {
    let current_segments = route_segments(&route.path);
    routes
        .iter()
        .filter(|candidate| candidate.path != route.path)
        .filter_map(|candidate| {
            let branch = candidate.annotation.branch.as_deref()?;
            let candidate_segments = route_segments(&candidate.path);
            (candidate_segments.len() < current_segments.len()
                && current_segments.starts_with(&candidate_segments))
            .then_some((candidate_segments.len(), branch))
        })
        .max_by_key(|(length, _)| *length)
        .map(|(_, branch)| branch)
}
