use dust_ir::{BuiltinType, TypeIr};

use crate::plugin::model::RouteParamSpec;

/// Generated decode statement and validation for one query parameter.
pub(super) struct QueryDecode {
    /// Dart expression assigned to the route parameter local.
    pub(super) expr: String,
    /// Condition that returns the not-found route when decoding failed.
    pub(super) invalid_condition: Option<String>,
}

/// Renders a Dart expression that encodes a route parameter for a URL segment.
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
        TypeIr::Named { name: ty_name, .. } if ty_name.as_ref() == "Uri" => {
            format!("{access}.toString()")
        }
        TypeIr::Named { name: ty_name, .. } if ty_name.as_ref() == "DateTime" => {
            format!("{access}.toIso8601String()")
        }
        TypeIr::Named {
            name: ty_name,
            args,
            ..
        } if ty_name.as_ref() == "List"
            && args.len() == 1
            && args[0].is_builtin(BuiltinType::Int) =>
        {
            format!("{access}.map((value) => value.toString()).toList(growable: false)")
        }
        TypeIr::Named {
            name: ty_name,
            args,
            ..
        } if ty_name.as_ref() == "List"
            && args.len() == 1
            && args[0].is_builtin(BuiltinType::String) =>
        {
            access
        }
        TypeIr::Named { .. } => format!("{access}.name"),
        _ => format!("{access}.toString()"),
    }
}

/// Renders a Dart expression that decodes one path segment.
pub(super) fn decode_path_expr(ty: &TypeIr, index: usize) -> String {
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
        } => format!("generatedRouteParseBool(segments[{index}])"),
        _ => "null".to_owned(),
    }
}

/// Renders a Dart expression and validation condition for one query parameter.
pub(super) fn decode_query(param: &RouteParamSpec) -> QueryDecode {
    let name = &param.name;
    let value = format!("uri.queryParameters['{name}']");
    let has_value = format!("uri.queryParameters.containsKey('{name}')");
    let all_values = format!("uri.queryParametersAll['{name}']");
    let has_all_values = format!("uri.queryParametersAll.containsKey('{name}')");
    match &param.ty {
        TypeIr::Builtin {
            kind: BuiltinType::String,
            nullable: true,
        } => QueryDecode {
            expr: value,
            invalid_condition: None,
        },
        TypeIr::Builtin {
            kind: BuiltinType::String,
            nullable: false,
        } => {
            if let Some(default) = &param.default_value_source {
                QueryDecode {
                    expr: format!("{value} ?? {default}"),
                    invalid_condition: None,
                }
            } else {
                QueryDecode {
                    expr: value,
                    invalid_condition: Some(format!("{name} == null")),
                }
            }
        }
        TypeIr::Builtin {
            kind: BuiltinType::Int,
            nullable: true,
        } => QueryDecode {
            expr: format!("{has_value} ? int.tryParse({value} ?? '') : null"),
            invalid_condition: Some(format!("{has_value} && {name} == null")),
        },
        TypeIr::Builtin {
            kind: BuiltinType::Int,
            nullable: false,
        } => decode_required_or_default(param, format!("int.tryParse({value} ?? '')"), name),
        TypeIr::Builtin {
            kind: BuiltinType::Double,
            nullable: true,
        } => QueryDecode {
            expr: format!("{has_value} ? double.tryParse({value} ?? '') : null"),
            invalid_condition: Some(format!("{has_value} && {name} == null")),
        },
        TypeIr::Builtin {
            kind: BuiltinType::Double,
            nullable: false,
        } => decode_required_or_default(param, format!("double.tryParse({value} ?? '')"), name),
        TypeIr::Builtin {
            kind: BuiltinType::Bool,
            nullable: true,
        } => QueryDecode {
            expr: format!("{has_value} ? generatedRouteParseBool({value}) : null"),
            invalid_condition: Some(format!("{has_value} && {name} == null")),
        },
        TypeIr::Builtin {
            kind: BuiltinType::Bool,
            nullable: false,
        } => decode_required_or_default(param, format!("generatedRouteParseBool({value})"), name),
        TypeIr::Named {
            name: ty_name,
            nullable: true,
            ..
        } if ty_name.as_ref() == "DateTime" => QueryDecode {
            expr: format!("{has_value} ? DateTime.tryParse({value} ?? '') : null"),
            invalid_condition: Some(format!("{has_value} && {name} == null")),
        },
        TypeIr::Named {
            name: ty_name,
            nullable: false,
            ..
        } if ty_name.as_ref() == "DateTime" => {
            decode_required_or_default(param, format!("DateTime.tryParse({value} ?? '')"), name)
        }
        TypeIr::Named {
            name: ty_name,
            nullable: true,
            ..
        } if ty_name.as_ref() == "Uri" => QueryDecode {
            expr: format!("{has_value} ? Uri.tryParse({value} ?? '') : null"),
            invalid_condition: Some(format!("{has_value} && {name} == null")),
        },
        TypeIr::Named {
            name: ty_name,
            nullable: false,
            ..
        } if ty_name.as_ref() == "Uri" => {
            decode_required_or_default(param, format!("Uri.tryParse({value} ?? '')"), name)
        }
        TypeIr::Named {
            name: ty_name,
            args,
            nullable: true,
        } if is_string_list(ty_name.as_ref(), args) => QueryDecode {
            expr: all_values,
            invalid_condition: None,
        },
        TypeIr::Named {
            name: ty_name,
            args,
            nullable: false,
        } if is_string_list(ty_name.as_ref(), args) => QueryDecode {
            expr: param
                .default_value_source
                .as_ref()
                .map_or(all_values.clone(), |default| {
                    format!("{all_values} ?? {default}")
                }),
            invalid_condition: param
                .default_value_source
                .is_none()
                .then(|| format!("{name} == null")),
        },
        TypeIr::Named {
            name: ty_name,
            args,
            nullable: true,
        } if is_int_list(ty_name.as_ref(), args) => QueryDecode {
            expr: format!("{has_all_values} ? generatedRouteParseIntList({all_values}) : null"),
            invalid_condition: Some(format!("{has_all_values} && {name} == null")),
        },
        TypeIr::Named {
            name: ty_name,
            args,
            nullable: false,
        } if is_int_list(ty_name.as_ref(), args) => QueryDecode {
            expr: param.default_value_source.as_ref().map_or_else(
                || format!("generatedRouteParseIntList({all_values})"),
                |default| {
                    format!(
                        "{has_all_values} ? generatedRouteParseIntList({all_values}) : {default}"
                    )
                },
            ),
            invalid_condition: Some(format!("{name} == null")),
        },
        TypeIr::Named {
            name: ty_name,
            nullable,
            ..
        } => {
            let parsed = format!("generatedRouteParseEnum({ty_name}.values, {value})");
            if *nullable {
                QueryDecode {
                    expr: format!("{has_value} ? {parsed} : null"),
                    invalid_condition: Some(format!("{has_value} && {name} == null")),
                }
            } else {
                decode_required_or_default(param, parsed, name)
            }
        }
        _ => QueryDecode {
            expr: "null".to_owned(),
            invalid_condition: Some("true".to_owned()),
        },
    }
}

/// Decodes a required query value, using the default only when the key is absent.
fn decode_required_or_default(
    param: &RouteParamSpec,
    parse_expr: String,
    name: &str,
) -> QueryDecode {
    let expr = param
        .default_value_source
        .as_ref()
        .map_or(parse_expr.clone(), |default| {
            format!("uri.queryParameters.containsKey('{name}') ? {parse_expr} : {default}")
        });
    QueryDecode {
        expr,
        invalid_condition: Some(format!("{name} == null")),
    }
}

/// Returns true for the repeated string query parameter shape.
fn is_string_list(name: &str, args: &[TypeIr]) -> bool {
    name == "List" && args.len() == 1 && args[0].is_builtin(BuiltinType::String)
}

/// Returns true for the repeated integer query parameter shape.
fn is_int_list(name: &str, args: &[TypeIr]) -> bool {
    name == "List" && args.len() == 1 && args[0].is_builtin(BuiltinType::Int)
}
