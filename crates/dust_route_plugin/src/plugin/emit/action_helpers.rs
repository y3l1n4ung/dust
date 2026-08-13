use dust_dart_emit::render_template;
use serde::Serialize;

use crate::plugin::model::{RouteParamSpec, RouteSpec, RouterSpec};

use super::formatting::dart_type;

/// Template context for one route action factory.
#[derive(Serialize)]
struct FactoryContext {
    /// Factory method signature.
    factory: String,
    /// Factory method body expression.
    body: String,
}

/// Renders route action factory methods for all routes.
pub(super) fn render_route_factories(spec: &RouterSpec) -> String {
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
