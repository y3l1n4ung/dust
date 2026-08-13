use std::collections::BTreeSet;

use dust_dart_emit::{dart_string_literal, render_template};
use serde::Serialize;

use crate::plugin::model::{GuardSpec, RouteParamSpec, RouteSpec, RouterSpec};

use super::{
    formatting::dart_type,
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

/// Template context for one route action factory.
#[derive(Serialize)]
struct FactoryContext {
    /// Factory method signature.
    factory: String,
    /// Factory method body expression.
    body: String,
}

/// Renders generated navigation helpers for guards and route actions.
pub(super) fn render_helpers(out: &mut String, spec: &RouterSpec) {
    out.push_str(&render_template(
        "route_helpers",
        include_str!("templates/route_helpers.jinja"),
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
        },
    ));
    out.push_str("\n\n");
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
    guard_constructor(guard_spec, &spec.router_class)
}

/// Renders a guard constructor call from a resolved guard spec.
fn guard_constructor(guard: &GuardSpec, router_class: &str) -> String {
    if guard.params.is_empty() {
        return format!("{}()", guard.class_name);
    }
    let args = guard
        .params
        .iter()
        .filter_map(|param| {
            param.inject_field.as_ref().map(|field| {
                let expr = format!("(router as {router_class}).{field}");
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

/// Renders route action factory methods for all routes.
fn render_route_factories(spec: &RouterSpec) -> String {
    let factories = spec
        .routes
        .iter()
        .map(|route| render_route_factory(route, &spec.route_action_class))
        .collect::<Vec<_>>()
        .join("\n\n");
    if factories.is_empty() {
        String::new()
    } else {
        format!("{factories}\n\n")
    }
}

/// Renders one route action factory method.
fn render_route_factory(route: &RouteSpec, route_action_class: &str) -> String {
    let route_ctor = format!("{}({})", route.route_class, render_route_args(route));
    let params = render_factory_params(route);
    let factory = format!(
        "{}<{}> {}({params})",
        route_action_class, route.result_type, route.name
    );
    let body = format!("{route_action_class}(_router, {route_ctor})");
    render_template(
        if factory.len() + body.len() + 7 <= 80 {
            "route_factory_inline"
        } else {
            "route_factory_multiline"
        },
        if factory.len() + body.len() + 7 <= 80 {
            include_str!("templates/route_factory_inline.jinja")
        } else {
            include_str!("templates/route_factory_multiline.jinja")
        },
        FactoryContext { factory, body },
    )
}

/// Renders factory parameters for a route action.
fn render_factory_params(route: &RouteSpec) -> String {
    let params = route
        .params
        .iter()
        .map(render_factory_param)
        .collect::<Vec<_>>()
        .join(", ");
    if route.params.iter().any(|param| param.is_named) {
        format!("{{{params}}}")
    } else {
        params
    }
}

/// Renders one route action factory parameter.
fn render_factory_param(param: &RouteParamSpec) -> String {
    let ty = dart_type(&param.ty);
    if param.is_path || (!param.ty.is_nullable() && !param.has_default) {
        format!("required {ty} {}", param.name)
    } else if let Some(default_value) = &param.default_value_source {
        format!("{ty} {} = {default_value}", param.name)
    } else {
        format!("{ty} {}", param.name)
    }
}

/// Renders arguments passed from a route action factory to a route class.
fn render_route_args(route: &RouteSpec) -> String {
    route
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.name))
        .collect::<Vec<_>>()
        .join(", ")
}
