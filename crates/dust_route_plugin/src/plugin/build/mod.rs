use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr};
use dust_plugin_api::SymbolPlan;

use super::{
    constants::{ROUTER, ROUTERS_ANALYSIS_KEY},
    model::{RouteSpec, RouterFieldSpec, RouterSpec},
    parse::parse_router_config,
};

/// Builds guard specs and router-field injections.
mod guards;
/// Builds local and workspace route specs.
mod routes;

use guards::build_guard_specs;
use routes::{build_route_spec, workspace_route_specs};

/// Builds the final router spec for a library containing the workspace router.
pub(crate) fn build_router_spec(
    library: &DartFileIr,
    plan: &SymbolPlan,
) -> Result<Option<RouterSpec>, Vec<Diagnostic>> {
    let router_classes = router_classes(library);
    let Some(router_class) = router_classes.first().copied() else {
        return Ok(None);
    };
    if router_classes.len() > 1 || workspace_router_count(plan) > 1 {
        return Err(vec![Diagnostic::error(
            "exactly one `@AppRouter` is allowed in a Dust route workspace",
        )]);
    }

    let router_config = router_class
        .configs
        .iter()
        .find(|config| config.symbol.0.rsplit("::").next() == Some(ROUTER));
    let router_annotation = parse_router_config(router_config);
    let mut routes = local_and_workspace_routes(library, plan);
    routes.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.name.cmp(&b.name)));

    if routes.is_empty() {
        return Err(vec![Diagnostic::error(format!(
            "router `{}` needs at least one `@AppRoute` page in the workspace for current route generation",
            router_class.name
        ))]);
    }

    validate_workspace_route_set(&routes)?;
    let initial_route_class = route_class_for_path(
        &routes,
        router_annotation.initial.as_deref(),
        &router_class.name,
        "initial",
    )?;
    let not_found_route_class = route_class_for_path(
        &routes,
        router_annotation.not_found.as_deref(),
        &router_class.name,
        "notFound",
    )?;
    validate_not_found_route(&routes, &not_found_route_class)?;

    let router_fields = router_fields(router_class);
    let refresh_listenable = discover_refresh_listenable(&router_fields)?;
    let guard_specs = build_guard_specs(library, plan, &routes, &router_fields)?;

    Ok(Some(RouterSpec {
        router_class: router_class.name.clone(),
        generated_base_class: format!("${}", router_class.name),
        initial_route_class,
        not_found_route_class: Some(not_found_route_class),
        refresh_listenable,
        guard_specs,
        routes,
    }))
}

/// Returns classes annotated with `@AppRouter` in the current library.
fn router_classes(library: &DartFileIr) -> Vec<&ClassIr> {
    library
        .classes
        .iter()
        .filter(|class| {
            class
                .configs
                .iter()
                .any(|config| config.symbol.0.rsplit("::").next() == Some(ROUTER))
        })
        .collect()
}

/// Merges local route specs with workspace route facts from other files.
fn local_and_workspace_routes(library: &DartFileIr, plan: &SymbolPlan) -> Vec<RouteSpec> {
    let mut routes = library
        .classes
        .iter()
        .filter_map(build_route_spec)
        .collect::<Vec<_>>();
    let local_pages = routes
        .iter()
        .map(|route| route.page_class.clone())
        .collect::<HashSet<_>>();
    routes.extend(workspace_route_specs(plan, &local_pages));
    routes
}

/// Counts discovered routers across the workspace analysis set.
fn workspace_router_count(plan: &SymbolPlan) -> usize {
    plan.workspace_string_set(ROUTERS_ANALYSIS_KEY)
        .unwrap_or_default()
        .len()
}

/// Validates duplicate route paths, names, and required query parameters.
fn validate_workspace_route_set(routes: &[RouteSpec]) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut paths = HashSet::new();
    let mut names = HashSet::new();
    for route in routes {
        if !paths.insert(route.path.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "duplicate route path `{}`",
                route.path
            )));
        }
        if !names.insert(route.name.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "duplicate route name `{}`",
                route.name
            )));
        }
        for param in &route.params {
            if !param.is_path && !param.ty.is_nullable() && !param.has_default {
                diagnostics.push(Diagnostic::error(format!(
                    "route query parameter `{}` on `{}` must be nullable or have a default value",
                    param.name, route.page_class
                )));
            }
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
                &sibling.path,
                &route.path,
                &parent,
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
                display_parent_path(
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
fn ambiguous_sibling_diagnostic(route_path: &str, sibling_path: &str, parent: &str) -> Diagnostic {
    Diagnostic::error(format!(
        "route path `{route_path}` conflicts with sibling `{sibling_path}`; static and dynamic segments under `{parent}` are ambiguous"
    ))
}

/// Iterates over non-empty slash-delimited path segments.
fn path_segments(path: &str) -> impl Iterator<Item = &str> {
    path.split('/').filter(|segment| !segment.is_empty())
}

/// Returns the parameter name for a dynamic path segment.
fn path_param_name(segment: &str) -> Option<&str> {
    segment.strip_prefix(':').filter(|name| !name.is_empty())
}

/// Renders parent path segments with a leading slash.
fn display_parent_path(segments: &[String]) -> String {
    if segments.is_empty() {
        "/".to_owned()
    } else {
        format!("/{}", segments.join("/"))
    }
}

/// Resolves an annotation path to the generated route class for router settings.
fn route_class_for_path(
    routes: &[RouteSpec],
    path: Option<&str>,
    router_class: &str,
    label: &str,
) -> Result<String, Vec<Diagnostic>> {
    let Some(path) = path else {
        return Err(vec![Diagnostic::error(format!(
            "router `{router_class}` requires `{label}` path"
        ))]);
    };
    routes
        .iter()
        .find(|route| route.path == path)
        .map(|route| route.route_class.clone())
        .ok_or_else(|| {
            vec![Diagnostic::error(format!(
                "router `{router_class}` {label} path `{path}` does not match any discovered `@AppRoute` path"
            ))]
        })
}

/// Ensures the not-found route remains unconditional.
fn validate_not_found_route(
    routes: &[RouteSpec],
    route_class: &str,
) -> Result<(), Vec<Diagnostic>> {
    let Some(route) = routes.iter().find(|route| route.route_class == route_class) else {
        return Ok(());
    };
    if route.annotation.guards_configured && !route.annotation.guards.is_empty() {
        return Err(vec![Diagnostic::error(format!(
            "notFound route `{}` must not declare guards",
            route.page_class
        ))]);
    }
    Ok(())
}

/// Extracts router fields available for refresh and guard injection.
fn router_fields(router_class: &ClassIr) -> Vec<RouterFieldSpec> {
    router_class
        .fields
        .iter()
        .filter_map(|field| {
            Some(RouterFieldSpec {
                name: field.name.clone(),
                type_name: field.ty.name()?.to_owned(),
            })
        })
        .collect()
}

/// Finds a single Listenable-like router field for refresh notifications.
fn discover_refresh_listenable(
    fields: &[RouterFieldSpec],
) -> Result<Option<String>, Vec<Diagnostic>> {
    let candidates = fields
        .iter()
        .filter(|field| is_listenable_type(&field.type_name))
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [field] => Ok(Some(field.name.clone())),
        _ => Err(vec![Diagnostic::error(
            "router has more than one Listenable-like field; keep exactly one refresh source",
        )]),
    }
}

/// Returns true when a router field type can refresh Navigator state.
fn is_listenable_type(name: &str) -> bool {
    matches!(name, "Listenable" | "ChangeNotifier" | "ValueNotifier") || name.ends_with("ViewModel")
}
