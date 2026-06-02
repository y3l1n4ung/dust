use std::collections::BTreeSet;

use crate::plugin::model::{RouteSpec, RouterSpec};

use super::{
    path::{is_path_prefix, route_segments},
    patterns::route_switch_pattern,
};

pub(super) fn render_restore_stack(out: &mut String, spec: &RouterSpec) {
    out.push_str("RouteStack<AppRoutePath> restoreAppRouteStack(AppRoutePath route) {\n");
    out.push_str("  return switch (route) {\n");
    for route in &spec.routes {
        let bound_params = restore_stack_bound_params(route, spec);
        out.push_str(&format!(
            "    {} => [\n",
            route_switch_pattern(route, Some(&bound_params))
        ));
        for entry in restore_stack_entries(route, spec) {
            out.push_str(&format!("      {entry},\n"));
        }
        out.push_str("    ],\n");
    }
    out.push_str("  };\n");
    out.push_str("}\n\n");
}

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
