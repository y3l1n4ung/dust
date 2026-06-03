use dust_dart_emit::render_template;
use dust_ir::TypeIr;
use serde::Serialize;

use crate::plugin::model::{RouteParamSpec, RouteSpec, RouterSpec};

use super::{formatting::dart_type, parser::encode_param_expr};

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
    let mut body = String::new();
    render_location_body(&mut body, route);
    render_template(
        "location_getter",
        include_str!("templates/location_getter.jinja"),
        LocationContext { body },
    )
}

fn render_location_body(out: &mut String, route: &RouteSpec) {
    if is_not_found_route(route) {
        out.push_str("    return '/404?path=${Uri.encodeComponent(path)}';\n");
        return;
    }

    let query_params = route
        .params
        .iter()
        .filter(|param| !param.is_path)
        .collect::<Vec<_>>();
    if !query_params.is_empty() {
        out.push_str("    final query = <String, String>{};\n");
        for param in query_params {
            let encode = encode_param_expr(&param.ty, &param.name);
            if param.ty.is_nullable() {
                out.push_str(&format!(
                    "    if ({0} != null) {{\n      query['{0}'] = {1};\n    }}\n",
                    param.name, encode
                ));
            } else if let Some(default_value) = &param.default_value_source {
                out.push_str(&format!(
                    "    if ({0} != {default_value}) {{\n      query['{0}'] = {1};\n    }}\n",
                    param.name, encode
                ));
            } else {
                out.push_str(&format!("    query['{}'] = {};\n", param.name, encode));
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
        out.push_str(&format!(
            "    return _routePath({segment_expr}, queryParameters: query.isEmpty ? null : query);\n"
        ));
    } else if segments.is_empty() {
        out.push_str("    return '/';\n");
    } else {
        let segment_expr = if inline_segments.len() <= 60 {
            inline_segments
        } else {
            multiline_segments
        };
        out.push_str(&format!("    return _routePath({segment_expr});\n"));
    }
}
