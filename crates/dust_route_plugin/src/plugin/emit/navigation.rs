use std::collections::BTreeSet;

use dust_dart_emit::{dart_string_literal, render_template};
use serde::Serialize;

use crate::plugin::model::{GuardSpec, RouterSpec};

use super::{
    action_helpers::render_route_factories,
    patterns::route_switch_pattern,
    shell::{effective_branch, effective_shell},
};

/// Template context for generated route navigation helpers.
#[derive(Serialize)]
struct HelpersContext {
    /// Generated route path base class.
    route_path_class: String,
    /// Generated route location helper function.
    route_location_function: String,
    /// Generated route auth helper function.
    route_requires_auth_function: String,
    /// Generated route branch helper function.
    route_branch_function: String,
    /// Generated route debug helper function.
    route_debug_info_function: String,
    /// Generated route guard helper function.
    route_guards_function: String,
    /// Generated BuildContext extension name.
    context_extension: String,
    /// Generated typed navigator helper class.
    navigator_class: String,
    /// Generated typed route action helper class.
    route_action_class: String,
    /// Rendered switch cases that instantiate route guards.
    guard_cases: String,
    /// Rendered switch cases that return branch names.
    branch_cases: String,
    /// Rendered switch cases that return route debug info.
    debug_cases: String,
    /// Rendered route action factory methods.
    factories: String,
    /// Generated router base class name.
    router_base_class: String,
}

/// Template context for one guard lookup case.
#[derive(Serialize)]
struct GuardCaseContext {
    /// Dart route pattern for the case.
    pattern: String,
    /// Rendered guard constructor list.
    guards: String,
}

/// Renders generated route path helpers.
pub(super) fn render_path_helpers(out: &mut String, spec: &RouterSpec) {
    out.push_str(&render_template(
        "route_path_helpers",
        include_str!("templates/route_path_helpers.jinja"),
        helpers_context(spec),
    ));
    out.push_str("\n\n");
}

/// Renders generated route metadata helper functions.
pub(super) fn render_metadata_helpers(out: &mut String, spec: &RouterSpec) {
    out.push_str(&render_template(
        "route_metadata_helpers",
        include_str!("templates/route_metadata_helpers.jinja"),
        helpers_context(spec),
    ));
    out.push_str("\n\n");
}

/// Renders generated navigation helpers for route actions.
pub(super) fn render_navigation_helpers(out: &mut String, spec: &RouterSpec) {
    out.push_str(&render_template(
        "route_navigation_helpers",
        include_str!("templates/route_navigation_helpers.jinja"),
        helpers_context(spec),
    ));
    out.push_str("\n\n");
}

/// Builds the shared helper template context.
fn helpers_context(spec: &RouterSpec) -> HelpersContext {
    HelpersContext {
        route_path_class: spec.route_path_class.clone(),
        route_location_function: spec.route_location_function.clone(),
        route_requires_auth_function: spec.route_requires_auth_function.clone(),
        route_branch_function: spec.route_branch_function.clone(),
        route_debug_info_function: spec.route_debug_info_function.clone(),
        route_guards_function: spec.route_guards_function.clone(),
        context_extension: spec.context_extension.clone(),
        navigator_class: spec.navigator_class.clone(),
        route_action_class: spec.route_action_class.clone(),
        guard_cases: render_guard_cases(spec),
        branch_cases: render_branch_cases(spec),
        debug_cases: render_debug_cases(spec),
        factories: render_route_factories(spec),
        router_base_class: spec.generated_base_class.clone(),
    }
}

/// Renders switch cases that return branch names for branched routes.
fn render_branch_cases(spec: &RouterSpec) -> String {
    let no_bindings = BTreeSet::new();
    spec.routes
        .iter()
        .filter_map(|route| {
            effective_branch(route, &spec.routes).map(|branch| {
                format!(
                    "    {} => {},\n",
                    route_switch_pattern(route, Some(&no_bindings)),
                    dart_string_literal(branch)
                )
            })
        })
        .collect::<String>()
}

/// Renders switch cases that return route debug metadata.
fn render_debug_cases(spec: &RouterSpec) -> String {
    let no_bindings = BTreeSet::new();
    spec.routes
        .iter()
        .map(|route| {
            let shell = effective_shell(route, &spec.routes)
                .map(dart_string_literal)
                .unwrap_or_else(|| "null".to_owned());
            let branch = effective_branch(route, &spec.routes)
                .map(dart_string_literal)
                .unwrap_or_else(|| "null".to_owned());
            let name = dart_string_literal(&route.name);
            format!(
                "    {} => const RouteDebugInfo(name: {}, shell: {}, branch: {}),\n",
                route_switch_pattern(route, Some(&no_bindings)),
                name,
                shell,
                branch
            )
        })
        .collect::<String>()
}

/// Renders switch cases that return guard instances for guarded routes.
fn render_guard_cases(spec: &RouterSpec) -> String {
    let cases = spec
        .routes
        .iter()
        .filter(|route| !route.annotation.guards.is_empty())
        .map(|route| {
            let guards = route
                .annotation
                .guards
                .iter()
                .map(|guard| render_guard_instance(guard, spec))
                .collect::<Vec<_>>()
                .join(", ");
            render_template(
                "route_guard_case",
                include_str!("templates/route_guard_case.jinja"),
                GuardCaseContext {
                    pattern: format!("{}()", route.route_class),
                    guards,
                },
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    if cases.is_empty() {
        String::new()
    } else {
        format!("{cases}\n")
    }
}

/// Renders a guard constructor expression, including router field injection.
fn render_guard_instance(guard: &str, spec: &RouterSpec) -> String {
    let Some(guard_spec) = spec
        .guard_specs
        .iter()
        .find(|candidate| candidate.class_name == guard)
    else {
        return format!("{guard}()");
    };
    guard_constructor(guard_spec)
}

/// Renders a guard constructor call from a resolved guard spec.
fn guard_constructor(guard: &GuardSpec) -> String {
    if guard.params.is_empty() {
        return format!("{}()", guard.class_name);
    }
    let args = guard
        .params
        .iter()
        .filter_map(|param| {
            param.inject_field.as_ref().map(|field| {
                let expr = format!("(router as dynamic).{field}");
                if param.is_named {
                    format!("{}: {expr}", param.name)
                } else {
                    expr
                }
            })
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{}({args})", guard.class_name)
}
