use dust_dart_emit::render_template;
use dust_ir::{BuiltinType, TypeIr};
use serde::Serialize;

use crate::plugin::model::{RouteParamSpec, RouteSpec, RouterSpec};

use super::parser_decode::{decode_path_expr, decode_query_expr};

use super::route_classes::is_not_found_route;

/// Template context for the generated URI parser.
#[derive(Serialize)]
struct ParserContext {
    /// Rendered route parsing cases.
    cases: String,
    /// Fallback route expression.
    fallback: String,
}

/// Template context for one route parser branch.
#[derive(Serialize)]
struct ParseCaseContext {
    /// Dart condition that matches the URI path shape.
    condition: String,
    /// Rendered path parameter decoder statements.
    decoders: String,
    /// Rendered null checks for failed decodes.
    null_checks: String,
    /// Route instance returned by the branch.
    route_instance: String,
}

/// Template context for one path parameter decoder.
#[derive(Serialize)]
struct ParseDecoderContext<'a> {
    /// Path parameter name.
    name: &'a str,
    /// Dart expression that decodes the path segment.
    expr: String,
}

/// Template context for one required decode null check.
#[derive(Serialize)]
struct ParseNullCheckContext<'a> {
    /// Path parameter name being checked.
    name: &'a str,
}

/// Renders the generated URI parser for all route specs.
pub(super) fn render_parser(out: &mut String, spec: &RouterSpec) {
    let fallback = spec
        .not_found_route_class
        .as_deref()
        .and_then(|class| {
            spec.routes
                .iter()
                .find(|route| route.route_class == class)
                .map(|route| {
                    if is_not_found_route(route) {
                        "NotFoundRoute(path: uri.toString())".to_owned()
                    } else {
                        route_constructor_with_fallback(route, "uri.toString()")
                    }
                })
        })
        .unwrap_or_else(|| format!("const {}()", spec.initial_route_class));
    out.push_str(&render_template(
        "route_parser",
        include_str!("templates/route_parser.jinja"),
        ParserContext {
            cases: join_rendered(spec.routes.iter().map(render_parse_case).collect()),
            fallback,
        },
    ));
    out.push_str("\n\n");
}

/// Renders a route constructor used by parser fallback logic.
fn route_constructor_with_fallback(route: &RouteSpec, fallback_expr: &str) -> String {
    if route.params.is_empty() {
        return format!("const {}()", route.route_class);
    }
    let args = route
        .params
        .iter()
        .map(|param| {
            let value = if param.is_path || (!param.ty.is_nullable() && !param.has_default) {
                fallback_value_for_param(param, fallback_expr)
            } else if let Some(default_value) = &param.default_value_source {
                default_value.clone()
            } else if param.ty.is_nullable() {
                "null".to_owned()
            } else {
                fallback_value_for_param(param, fallback_expr)
            };
            format!("{}: {value}", param.name)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{}({args})", route.route_class)
}

/// Returns a conservative fallback value for a required route parameter.
fn fallback_value_for_param(param: &RouteParamSpec, fallback_expr: &str) -> String {
    match &param.ty {
        TypeIr::Builtin {
            kind: BuiltinType::String,
            ..
        } => fallback_expr.to_owned(),
        TypeIr::Builtin {
            kind: BuiltinType::Int,
            ..
        } => "0".to_owned(),
        TypeIr::Builtin {
            kind: BuiltinType::Double,
            ..
        } => "0".to_owned(),
        TypeIr::Builtin {
            kind: BuiltinType::Bool,
            ..
        } => "false".to_owned(),
        _ => "null".to_owned(),
    }
}

/// Renders one URI parser case for a route.
fn render_parse_case(route: &RouteSpec) -> String {
    if is_not_found_route(route) {
        return render_template(
            "route_parse_not_found_case",
            include_str!("templates/route_parse_not_found_case.jinja"),
            (),
        );
    }

    let path_segments = route
        .path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let mut conditions = vec![if path_segments.is_empty() {
        "segments.isEmpty".to_owned()
    } else {
        format!("segments.length == {}", path_segments.len())
    }];
    for (index, segment) in path_segments.iter().enumerate() {
        if !segment.starts_with(':') {
            conditions.push(format!("segments[{index}] == '{segment}'"));
        }
    }
    let mut decoders = Vec::new();
    let mut null_checks = Vec::new();
    for (index, segment) in path_segments.iter().enumerate() {
        if let Some(name) = segment.strip_prefix(':') {
            let Some(param) = route.params.iter().find(|param| param.name == name) else {
                continue;
            };
            decoders.push(render_template(
                "route_parse_decoder",
                include_str!("templates/route_parse_decoder.jinja"),
                ParseDecoderContext {
                    name,
                    expr: decode_path_expr(&param.ty, index),
                },
            ));
            if !matches!(
                param.ty,
                TypeIr::Builtin {
                    kind: BuiltinType::String,
                    ..
                }
            ) {
                null_checks.push(render_template(
                    "route_parse_null_check",
                    include_str!("templates/route_parse_null_check.jinja"),
                    ParseNullCheckContext { name },
                ));
            }
        }
    }
    let mut args = Vec::new();
    for param in &route.params {
        if param.is_path {
            args.push(format!("{}: {}", param.name, param.name));
        } else {
            args.push(format!("{}: {}", param.name, decode_query_expr(param)));
        }
    }
    render_template(
        "route_parse_case",
        include_str!("templates/route_parse_case.jinja"),
        ParseCaseContext {
            condition: condition_expr(&conditions),
            decoders: join_chunks(decoders),
            null_checks: join_chunks(null_checks),
            route_instance: route_instance_expr(&route.route_class, &args),
        },
    )
}

/// Renders a readable Dart boolean expression from parser conditions.
fn condition_expr(conditions: &[String]) -> String {
    let inline = conditions.join(" && ");
    if inline.len() <= 76 {
        return inline;
    }
    format!("\n    {}\n  ", conditions.join(" &&\n    "))
}

/// Renders a route instance expression, wrapping long argument lists.
fn route_instance_expr(route_class: &str, args: &[String]) -> String {
    if args.is_empty() {
        return format!("const {route_class}()");
    }
    let inline = format!("{route_class}({})", args.join(", "));
    if inline.len() <= 60 {
        inline
    } else {
        format!("{route_class}(\n      {},\n    )", args.join(",\n      "))
    }
}

/// Joins generated chunks with a trailing newline when non-empty.
fn join_chunks(chunks: Vec<String>) -> String {
    if chunks.is_empty() {
        return String::new();
    }
    format!("{}\n", chunks.join("\n"))
}

/// Joins rendered parser cases with a trailing newline when non-empty.
fn join_rendered(chunks: Vec<String>) -> String {
    if chunks.is_empty() {
        String::new()
    } else {
        format!("{}\n", chunks.join("\n"))
    }
}
