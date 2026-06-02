use dust_ir::{BuiltinType, TypeIr};

use crate::plugin::model::{RouteParamSpec, RouteSpec, RouterSpec};

use super::route_classes::is_not_found_route;

pub(super) fn render_parser(out: &mut String, spec: &RouterSpec) {
    out.push_str("AppRoutePath parseAppRoute(Uri uri) {\n");
    out.push_str("  final segments = uri.pathSegments;\n\n");
    for route in &spec.routes {
        render_parse_case(out, route);
    }
    out.push_str("  return _notFoundRoute(uri);\n");
    out.push_str("}\n\n");
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
    out.push_str(&format!(
        "AppRoutePath _notFoundRoute(Uri uri) => {fallback};\n\n"
    ));
    out.push_str("bool? _parseBool(String? value) {\n");
    out.push_str("  return switch (value) {\n");
    out.push_str("    'true' || '1' => true,\n");
    out.push_str("    'false' || '0' => false,\n");
    out.push_str("    null || '' => null,\n");
    out.push_str("    _ => () {\n");
    out.push_str("      assert(\n");
    out.push_str("        false,\n");
    out.push_str("        '_parseBool: unrecognised value \"$value\", treating as null',\n");
    out.push_str("      );\n");
    out.push_str("      return null;\n");
    out.push_str("    }(),\n");
    out.push_str("  };\n");
    out.push_str("}\n\n");
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

fn render_parse_case(out: &mut String, route: &RouteSpec) {
    if is_not_found_route(route) {
        out.push_str("  if (segments.length == 1 && segments[0] == '404') {\n");
        out.push_str("    return NotFoundRoute(path: uri.queryParameters['path'] ?? '');\n");
        out.push_str("  }\n\n");
        return;
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
    out.push_str(&format!("  if ({}) {{\n", conditions.join(" && ")));
    for (index, segment) in path_segments.iter().enumerate() {
        if let Some(name) = segment.strip_prefix(':') {
            let Some(param) = route.params.iter().find(|param| param.name == name) else {
                continue;
            };
            out.push_str(&format!(
                "    final {name} = {};\n",
                decode_path_expr(&param.ty, index)
            ));
            if !matches!(
                param.ty,
                TypeIr::Builtin {
                    kind: BuiltinType::String,
                    ..
                }
            ) {
                out.push_str(&format!("    if ({name} == null) {{\n"));
                out.push_str("      return _notFoundRoute(uri);\n");
                out.push_str("    }\n");
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
    out.push_str(&format!(
        "    return {}({});\n",
        route.route_class,
        args.join(", ")
    ));
    out.push_str("  }\n\n");
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
