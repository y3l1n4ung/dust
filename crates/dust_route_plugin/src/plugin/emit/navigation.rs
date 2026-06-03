use dust_dart_emit::render_template;
use serde::Serialize;

use crate::plugin::model::{RouteParamSpec, RouteSpec, RouterSpec};

use super::formatting::{dart_type, upper_camel_identifier};

#[derive(Serialize)]
struct HelpersContext {
    guard_cases: String,
    factories: String,
}

#[derive(Serialize)]
struct GuardCaseContext {
    pattern: String,
    guards: String,
}

#[derive(Serialize)]
struct FactoryContext {
    factory: String,
    body: String,
}

pub(super) fn render_helpers(out: &mut String, spec: &RouterSpec) {
    out.push_str(&render_template(
        "route_helpers",
        include_str!("templates/route_helpers.jinja"),
        HelpersContext {
            guard_cases: render_guard_cases(spec),
            factories: spec.routes.iter().map(render_route_factory).collect(),
        },
    ));
    out.push_str("\n\n");
}

fn render_guard_cases(spec: &RouterSpec) -> String {
    spec.routes
        .iter()
        .filter(|route| !route.annotation.guards.is_empty())
        .map(|route| {
            let guards = route
                .annotation
                .guards
                .iter()
                .map(|guard| format!("router.create{}()", upper_camel_identifier(guard)))
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
        .collect()
}

fn render_route_factory(route: &RouteSpec) -> String {
    let route_ctor = format!("{}({})", route.route_class, render_route_args(route));
    let params = render_factory_params(route);
    let factory = format!("RouteNavigation<AppRoutePath> {}({params})", route.name);
    let body = format!("RouteNavigation(_router, {route_ctor})");
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

fn render_route_args(route: &RouteSpec) -> String {
    route
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.name))
        .collect::<Vec<_>>()
        .join(", ")
}
