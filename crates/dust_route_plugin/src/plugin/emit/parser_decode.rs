use dust_ir::{BuiltinType, TypeIr};

use crate::plugin::model::RouteParamSpec;

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
        } => format!("_parseBool(segments[{index}])"),
        _ => "null".to_owned(),
    }
}

pub(super) fn decode_query_expr(param: &RouteParamSpec) -> String {
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
