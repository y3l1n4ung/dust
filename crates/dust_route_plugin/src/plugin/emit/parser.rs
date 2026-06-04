use dust_dart_emit::render_template;
use dust_ir::{BuiltinType, TypeIr};
use serde::Serialize;

use crate::plugin::model::{RouteParamSpec, RouteSpec, RouterSpec};

use super::route_classes::is_not_found_route;

#[derive(Serialize)]
struct ParserContext {
    cases: String,
    fallback: String,
}

#[derive(Serialize)]
struct ParseCaseContext {
    condition: String,
    decoders: String,
    null_checks: String,
    route_instance: String,
}

#[derive(Serialize)]
struct ParseDecoderContext<'a> {
    name: &'a str,
    expr: String,
}

#[derive(Serialize)]
struct ParseNullCheckContext<'a> {
    name: &'a str,
}

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
            condition: conditions.join(" && "),
            decoders: join_chunks(decoders),
            null_checks: join_chunks(null_checks),
            route_instance: format!("{}({})", route.route_class, args.join(", ")),
        },
    )
}

fn join_chunks(chunks: Vec<String>) -> String {
    if chunks.is_empty() {
        return String::new();
    }
    format!("{}\n", chunks.join("\n"))
}

fn join_rendered(chunks: Vec<String>) -> String {
    if chunks.is_empty() {
        String::new()
    } else {
        format!("{}\n", chunks.join("\n"))
    }
}

pub(super) fn encode_param_expr(ty: &TypeIr, name: &str) -> String {
    let access = if ty.is_nullable() {
        format!("{name}!")
    } else {
        name.to_owned()
    };
    match ty {
        TypeIr::Builtin {
            kind: BuiltinType::String,
            ..
        } => access,
        _ => format!("{access}.toString()"),
    }
}

fn decode_path_expr(ty: &TypeIr, index: usize) -> String {
    match ty {
        TypeIr::Builtin {
            kind: BuiltinType::String,
            ..
        } => format!("segments[{index}]"),
        TypeIr::Builtin {
            kind: BuiltinType::Int,
            ..
        } => format!("int.tryParse(segments[{index}])"),
        TypeIr::Builtin {
            kind: BuiltinType::Double,
            ..
        } => format!("double.tryParse(segments[{index}])"),
        TypeIr::Builtin {
            kind: BuiltinType::Bool,
            ..
        } => format!("_parseBool(segments[{index}])"),
        _ => "null".to_owned(),
    }
}

fn decode_query_expr(param: &RouteParamSpec) -> String {
    let name = &param.name;
    match &param.ty {
        TypeIr::Builtin {
            kind: BuiltinType::String,
            nullable: true,
        } => format!("uri.queryParameters['{name}']"),
        TypeIr::Builtin {
            kind: BuiltinType::String,
            nullable: false,
        } => format!(
            "uri.queryParameters['{name}'] ?? {}",
            param.default_value_source.as_deref().unwrap_or("''")
        ),
        TypeIr::Builtin {
            kind: BuiltinType::Int,
            nullable: true,
        } => format!("int.tryParse(uri.queryParameters['{name}'] ?? '')"),
        TypeIr::Builtin {
            kind: BuiltinType::Int,
            nullable: false,
        } => format!(
            "int.tryParse(uri.queryParameters['{name}'] ?? '') ?? {}",
            param.default_value_source.as_deref().unwrap_or("0")
        ),
        TypeIr::Builtin {
            kind: BuiltinType::Double,
            nullable: true,
        } => format!("double.tryParse(uri.queryParameters['{name}'] ?? '')"),
        TypeIr::Builtin {
            kind: BuiltinType::Double,
            nullable: false,
        } => format!(
            "double.tryParse(uri.queryParameters['{name}'] ?? '') ?? {}",
            param.default_value_source.as_deref().unwrap_or("0")
        ),
        TypeIr::Builtin {
            kind: BuiltinType::Bool,
            nullable: true,
        } => format!("_parseBool(uri.queryParameters['{name}'])"),
        TypeIr::Builtin {
            kind: BuiltinType::Bool,
            nullable: false,
        } => format!(
            "_parseBool(uri.queryParameters['{name}']) ?? {}",
            param.default_value_source.as_deref().unwrap_or("false")
        ),
        _ => "null".to_owned(),
    }
}
