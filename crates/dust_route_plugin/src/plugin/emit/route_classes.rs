use dust_dart_emit::render_template;
use dust_ir::TypeIr;
use serde::Serialize;

use crate::plugin::model::{RouteParamSpec, RouteSpec, RouterSpec};

use super::{formatting::dart_type, parser_decode::encode_param_expr};

#[derive(Serialize)]
struct RouteClassContext<'a> {
    route_class: &'a str,
    constructor: String,
    fields: String,
    location: String,
    requires_auth: String,
}

#[derive(Serialize)]
struct LocationContext {
    body: String,
}

#[derive(Serialize)]
struct LocationQueryContext {
    name: String,
    encode: String,
    default_value: String,
}

#[derive(Serialize)]
struct LocationReturnContext {
    segments: String,
}

pub(super) fn render_route_classes(out: &mut String, spec: &RouterSpec) {
    out.push_str(&render_template(
        "app_route_base",
        include_str!("templates/app_route_base.jinja"),
        (),
    ));
    out.push_str("\n\n");

    for route in &spec.routes {
        let constructor_params = route
            .params
            .iter()
            .map(render_constructor_param)
            .collect::<Vec<_>>()
            .join(", ");
        let constructor = if constructor_params.is_empty() {
            format!("const {}();", route.route_class)
        } else {
            format!("const {}({{{constructor_params}}});", route.route_class)
        };
        out.push_str(&render_template(
            "route_class",
            include_str!("templates/route_class.jinja"),
            RouteClassContext {
                route_class: &route.route_class,
                constructor,
                fields: render_route_fields(route),
                location: render_location_getter(route),
                requires_auth: if route.annotation.guards_configured
                    && route.annotation.guards.is_empty()
                    || spec.not_found_route_class.as_deref() == Some(route.route_class.as_str())
                {
                    "\n\n  @override\n  bool get requiresAuth => false;".to_owned()
                } else {
                    String::new()
                },
            },
        ));
        out.push_str("\n\n");
    }
}

fn render_route_fields(route: &RouteSpec) -> String {
    let fields = route
        .params
        .iter()
        .map(|param| format!("  final {} {};", dart_type(&param.ty), param.name))
        .collect::<Vec<_>>()
        .join("\n");
    if fields.is_empty() {
        String::new()
    } else {
        format!("{fields}\n\n")
    }
}
fn render_constructor_param(param: &RouteParamSpec) -> String {
    if param.is_path || (!param.ty.is_nullable() && !param.has_default) {
        format!("required this.{}", param.name)
    } else if let Some(default_value) = &param.default_value_source {
        format!("this.{} = {default_value}", param.name)
    } else {
        format!("this.{}", param.name)
    }
}

pub(super) fn is_not_found_route(route: &RouteSpec) -> bool {
    route.route_class == "NotFoundRoute" && route.params.iter().any(|param| param.name == "path")
}
fn render_location_getter(route: &RouteSpec) -> String {
    render_template(
        "location_getter",
        include_str!("templates/location_getter.jinja"),
        LocationContext {
            body: render_location_body(route),
        },
    )
}

fn render_location_body(route: &RouteSpec) -> String {
    if is_not_found_route(route) {
        return format!(
            "{}\n",
            render_template(
                "location_not_found_body",
                include_str!("templates/location_not_found_body.jinja"),
                (),
            )
        );
    }

    let query_params = route
        .params
        .iter()
        .filter(|param| !param.is_path)
        .collect::<Vec<_>>();
    let mut body = Vec::new();
    if !query_params.is_empty() {
        body.push(render_template(
            "location_query_init",
            include_str!("templates/location_query_init.jinja"),
            (),
        ));
        for param in query_params {
            let encode = encode_param_expr(&param.ty, &param.name);
            let context = LocationQueryContext {
                name: param.name.clone(),
                encode,
                default_value: param.default_value_source.clone().unwrap_or_default(),
            };
            if param.ty.is_nullable() {
                body.push(render_template(
                    "location_query_nullable",
                    include_str!("templates/location_query_nullable.jinja"),
                    context,
                ));
            } else if param.default_value_source.is_some() {
                body.push(render_template(
                    "location_query_default",
                    include_str!("templates/location_query_default.jinja"),
                    context,
                ));
            } else {
                body.push(render_template(
                    "location_query_required",
                    include_str!("templates/location_query_required.jinja"),
                    context,
                ));
            }
        }
    }
    let segments = route
        .path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            if let Some(name) = segment.strip_prefix(':') {
                encode_param_expr(
                    route
                        .params
                        .iter()
                        .find(|param| param.name == name)
                        .map(|param| &param.ty)
                        .unwrap_or(&TypeIr::string()),
                    name,
                )
            } else {
                format!("'{segment}'")
            }
        })
        .collect::<Vec<_>>();
    let inline_segments = format!("[{}]", segments.join(", "));
    let multiline_segments = format!("[\n      {},\n    ]", segments.join(",\n      "));
    if route.params.iter().any(|param| !param.is_path) {
        let segment_expr = if inline_segments.len() <= 60 {
            inline_segments
        } else {
            multiline_segments
        };
        body.push(render_template(
            "location_return_query",
            include_str!("templates/location_return_query.jinja"),
            LocationReturnContext {
                segments: segment_expr,
            },
        ));
    } else if segments.is_empty() {
        body.push(render_template(
            "location_return_root",
            include_str!("templates/location_return_root.jinja"),
            (),
        ));
    } else {
        let segment_expr = if inline_segments.len() <= 60 {
            inline_segments
        } else {
            multiline_segments
        };
        body.push(render_template(
            "location_return_path",
            include_str!("templates/location_return_path.jinja"),
            LocationReturnContext {
                segments: segment_expr,
            },
        ));
    }
    format!("{}\n", body.join("\n"))
}
