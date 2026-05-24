use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{BuiltinType, MethodParamIr, TypeIr};

use crate::plugin::util::{is_response_body_type, type_name_is};

pub(super) fn render_type(ty: &TypeIr) -> String {
    DYNAMIC_TYPES.render(ty)
}

pub(super) fn render_non_nullable_type(ty: &TypeIr) -> String {
    DYNAMIC_TYPES.render_non_nullable(ty)
}

pub(super) fn render_fetch_type(ty: &TypeIr) -> String {
    if is_void_type(ty) {
        return "void".to_owned();
    }
    if is_response_body_type(ty) {
        return "ResponseBody".to_owned();
    }
    match ty {
        TypeIr::Dynamic => "dynamic".to_owned(),
        TypeIr::Builtin {
            kind: BuiltinType::String,
            ..
        } => "String".to_owned(),
        TypeIr::Builtin { .. } => "dynamic".to_owned(),
        TypeIr::Named { args, .. } if type_name_is(ty, "List") && args.len() == 1 => {
            "List<dynamic>".to_owned()
        }
        TypeIr::Named { .. } => "Map<String, dynamic>".to_owned(),
        TypeIr::Function { .. } | TypeIr::Record { .. } | TypeIr::Unknown => "dynamic".to_owned(),
    }
}

pub(super) fn render_body_value(param: &MethodParamIr) -> String {
    if uses_direct_body_value(&param.ty) {
        param.name.clone()
    } else {
        format!("{}.toJson()", param.name)
    }
}

pub(super) fn render_decode_expr(data_expr: &str, ty: &TypeIr) -> String {
    if ty.is_nullable() {
        format!(
            "{0} == null\n    ? null\n    : {1}",
            data_expr,
            render_decode_expr_nonnull(data_expr, ty)
        )
    } else {
        render_decode_expr_nonnull(data_expr, ty)
    }
}

pub(super) fn render_decode_expr_nonnull(data_expr: &str, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Dynamic | TypeIr::Unknown => data_expr.to_owned(),
        TypeIr::Builtin { kind, .. } => match kind {
            BuiltinType::String => format!("{data_expr} as String"),
            BuiltinType::Int => format!("({data_expr} as num).toInt()"),
            BuiltinType::Bool => format!("{data_expr} as bool"),
            BuiltinType::Double => format!("({data_expr} as num).toDouble()"),
            BuiltinType::Num => format!("{data_expr} as num"),
            BuiltinType::Object => format!("{data_expr} as Object"),
        },
        TypeIr::Named { .. } if type_name_is(ty, "void") => "null".to_owned(),
        TypeIr::Named { .. } if is_response_body_type(ty) => data_expr.to_owned(),
        TypeIr::Named { args, .. } if type_name_is(ty, "List") && args.len() == 1 => {
            format!(
                "({0} as List<dynamic>)\n    .map((item) => {1})\n    .toList()",
                data_expr,
                render_decode_expr("item", &args[0])
            )
        }
        TypeIr::Named { .. } if type_name_is(ty, "Map") => {
            format!("{data_expr} as {}", render_non_nullable_type(ty))
        }
        TypeIr::Named { .. } => format!(
            "{}.fromJson({} as Map<String, dynamic>)",
            render_non_nullable_type(ty),
            data_expr
        ),
        TypeIr::Function { .. } | TypeIr::Record { .. } => data_expr.to_owned(),
    }
}

pub(super) fn needs_isolate_helper(ty: &TypeIr) -> bool {
    match ty {
        TypeIr::Named { args, .. } if type_name_is(ty, "List") && args.len() == 1 => {
            needs_isolate_helper(&args[0])
        }
        TypeIr::Named { .. } if type_name_is(ty, "void") => false,
        TypeIr::Named { .. } if type_name_is(ty, "Map") => false,
        TypeIr::Named { .. } if is_response_body_type(ty) => false,
        TypeIr::Named { .. } => true,
        TypeIr::Builtin { .. } | TypeIr::Dynamic | TypeIr::Unknown => false,
        TypeIr::Function { .. } | TypeIr::Record { .. } => false,
    }
}

pub(super) fn is_void_type(ty: &TypeIr) -> bool {
    ty.is_named("void")
}

pub(super) fn uses_direct_body_value(ty: &TypeIr) -> bool {
    match ty {
        TypeIr::Dynamic | TypeIr::Unknown => true,
        TypeIr::Builtin { .. } => true,
        TypeIr::Named { .. } if type_name_is(ty, "List") || type_name_is(ty, "Map") => true,
        TypeIr::Named { .. } if type_name_is(ty, "Object") || type_name_is(ty, "void") => true,
        TypeIr::Named { .. } | TypeIr::Function { .. } | TypeIr::Record { .. } => false,
    }
}
