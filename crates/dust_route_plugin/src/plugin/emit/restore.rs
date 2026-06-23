use std::collections::BTreeSet;

use dust_dart_emit::render_template;
use serde::Serialize;

use crate::plugin::model::{RouteSpec, RouterSpec};

use super::{
    path::{is_path_prefix, route_segments},
    patterns::route_switch_pattern,
};

/// Template context for generated restore-stack switch cases.
#[derive(Serialize)]
struct RestoreStackContext {
    /// Rendered restore cases.
    cases: String,
}

/// Template context for one restore-stack case.
#[derive(Serialize)]
struct RestoreCaseContext {
    /// Dart route switch pattern.
    pattern: String,
    /// Rendered stack entries for the route.
    entries: String,
}

/// Template context for one restored stack entry.
#[derive(Serialize)]
struct RestoreEntryContext<'a> {
    /// Route expression placed in the restored stack.
    entry: &'a str,
}

/// Renders route stack restoration helpers.
pub(super) fn render_restore_stack(out: &mut String, spec: &RouterSpec) {
    let cases = spec
        .routes
        .iter()
        .map(|route| render_restore_case(route, spec))
        .collect::<Vec<_>>()
        .join("\n");
    out.push_str(&render_template(
        "restore_stack",
        include_str!("templates/restore_stack.jinja"),
        RestoreStackContext { cases },
    ));
    out.push_str("\n\n");
}

/// Renders one restore-stack switch case.
fn render_restore_case(route: &RouteSpec, spec: &RouterSpec) -> String {
    let bound_params = restore_stack_bound_params(route, spec);
    let entries = restore_stack_entries(route, spec)
        .iter()
        .map(|entry| {
            render_template(
                "restore_entry",
                include_str!("templates/restore_entry.jinja"),
                RestoreEntryContext { entry },
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    render_template(
        "restore_case",
        include_str!("templates/restore_case.jinja"),
        RestoreCaseContext {
            pattern: route_switch_pattern(route, Some(&bound_params)),
            entries,
        },
    )
}

/// Computes the stack entries restored for a target route.
fn restore_stack_entries(route: &RouteSpec, spec: &RouterSpec) -> Vec<String> {
    if route.route_class == spec.initial_route_class {
        return vec!["route".to_owned()];
    }

    let mut entries = Vec::new();
    if let Some(initial) = spec
        .routes
        .iter()
        .find(|candidate| candidate.route_class == spec.initial_route_class)
        && let Some(expr) = route_constructor_from_target(initial, route)
    {
        entries.push(expr);
    }

    let mut parents = spec
        .routes
        .iter()
        .filter(|candidate| candidate.route_class != spec.initial_route_class)
        .filter(|candidate| candidate.route_class != route.route_class)
        .filter(|candidate| is_path_prefix(&candidate.path, &route.path))
        .collect::<Vec<_>>();
    parents.sort_by_key(|candidate| route_segments(&candidate.path).len());

    for parent in parents {
        if let Some(expr) = route_constructor_from_target(parent, route) {
            entries.push(expr);
        }
    }

    entries.push("route".to_owned());
    entries
}

/// Returns route parameters that must be bound by a restore pattern.
fn restore_stack_bound_params(route: &RouteSpec, spec: &RouterSpec) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    if route.route_class == spec.initial_route_class {
        return names;
    }

    for parent in spec.routes.iter().filter(|candidate| {
        candidate.route_class == spec.initial_route_class
            || (candidate.route_class != route.route_class
                && is_path_prefix(&candidate.path, &route.path))
    }) {
        if route_constructor_from_target(parent, route).is_some() {
            names.extend(parent.params.iter().map(|param| param.name.clone()));
        }
    }
    names
}
/// Builds a route constructor for an ancestor route using target route params.
fn route_constructor_from_target(route: &RouteSpec, target: &RouteSpec) -> Option<String> {
    if route.route_class == target.route_class {
        return Some("route".to_owned());
    }
    if route.params.is_empty() {
        return Some(format!("const {}()", route.route_class));
    }

    let args = route
        .params
        .iter()
        .filter_map(|param| {
            if target
                .params
                .iter()
                .any(|candidate| candidate.name == param.name)
            {
                Some(Some(format!("{}: {}", param.name, param.name)))
            } else if param.ty.is_nullable() || param.has_default {
                None
            } else {
                Some(None)
            }
        })
        .collect::<Option<Vec<_>>>()?;

    if args.is_empty() {
        Some(format!("const {}()", route.route_class))
    } else {
        Some(format!("{}({})", route.route_class, args.join(", ")))
    }
}
