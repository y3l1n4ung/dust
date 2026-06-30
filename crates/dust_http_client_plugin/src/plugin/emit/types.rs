use dust_dart_emit::{
    DART_BOOL, DART_DYNAMIC, DART_LIST, DART_MAP, DART_NUM, DART_OBJECT, DART_RESPONSE_BODY,
    DART_STRING, DART_VOID, DYNAMIC_TYPES,
};
use dust_ir::{BuiltinType, MethodParamIr, TypeIr};

use crate::plugin::util::{is_response_body_type, type_name_is};

/// Renders a Dart type using the HTTP plugin's dynamic-type policy.
pub(super) fn render_type(ty: &TypeIr) -> String {
    DYNAMIC_TYPES.render(ty)
}

/// Renders a non-nullable Dart type for decode helper signatures.
pub(super) fn render_non_nullable_type(ty: &TypeIr) -> String {
    DYNAMIC_TYPES.render_non_nullable(ty)
}

/// Renders the type argument passed to `_dio.fetch`.
pub(super) fn render_fetch_type(ty: &TypeIr) -> String {
    if is_void_type(ty) {
        return DART_VOID.to_owned();
    }
    if is_response_body_type(ty) {
        return DART_RESPONSE_BODY.to_owned();
    }
    match ty {
        TypeIr::Dynamic => DART_DYNAMIC.to_owned(),
        TypeIr::Builtin {
            kind: BuiltinType::String,
            ..
        } => DART_STRING.to_owned(),
        TypeIr::Builtin { .. } => DART_DYNAMIC.to_owned(),
        TypeIr::Named { args, .. } if type_name_is(ty, DART_LIST) && args.len() == 1 => {
            format!("{DART_LIST}<{DART_DYNAMIC}>")
        }
        TypeIr::Named { .. } => format!("{DART_MAP}<{DART_STRING}, {DART_DYNAMIC}>"),
        TypeIr::Function { .. } | TypeIr::Record { .. } | TypeIr::Unknown => {
            DART_DYNAMIC.to_owned()
        }
    }
}

/// Renders the Dart body expression for a `@Body` parameter.
pub(super) fn render_body_value(param: &MethodParamIr) -> String {
    if uses_direct_body_value(&param.ty) {
        param.name.clone()
    } else {
        format!("{}.toJson()", param.name)
    }
}

/// Renders a Dart decode expression that preserves nullable payloads.
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

/// Renders a Dart decode expression for a known non-null payload value.
pub(super) fn render_decode_expr_nonnull(data_expr: &str, ty: &TypeIr) -> String {
    match ty {
        TypeIr::Dynamic | TypeIr::Unknown => data_expr.to_owned(),
        TypeIr::Builtin { kind, .. } => match kind {
            BuiltinType::String => format!("{data_expr} as {DART_STRING}"),
            BuiltinType::Int => format!("({data_expr} as {DART_NUM}).toInt()"),
            BuiltinType::Bool => format!("{data_expr} as {DART_BOOL}"),
            BuiltinType::Double => format!("({data_expr} as {DART_NUM}).toDouble()"),
            BuiltinType::Num => format!("{data_expr} as {DART_NUM}"),
            BuiltinType::Object => format!("{data_expr} as {DART_OBJECT}"),
        },
        TypeIr::Named { .. } if type_name_is(ty, DART_VOID) => "null".to_owned(),
        TypeIr::Named { .. } if is_response_body_type(ty) => data_expr.to_owned(),
        TypeIr::Named { args, .. } if type_name_is(ty, DART_LIST) && args.len() == 1 => {
            format!(
                "({0} as {DART_LIST}<{DART_DYNAMIC}>)\n    .map((item) => {1})\n    .toList()",
                data_expr,
                render_decode_expr("item", &args[0])
            )
        }
        TypeIr::Named { .. } if type_name_is(ty, DART_MAP) => {
            format!("{data_expr} as {}", render_non_nullable_type(ty))
        }
        TypeIr::Named { .. } => format!(
            "{}.fromJson({} as {DART_MAP}<{DART_STRING}, {DART_DYNAMIC}>)",
            render_non_nullable_type(ty),
            data_expr
        ),
        TypeIr::Function { .. } | TypeIr::Record { .. } => data_expr.to_owned(),
    }
}

/// Returns true when decoding should be moved to a generated isolate helper.
pub(crate) fn needs_isolate_helper(ty: &TypeIr) -> bool {
    match ty {
        TypeIr::Named { args, .. } if type_name_is(ty, DART_LIST) && args.len() == 1 => {
            needs_isolate_helper(&args[0])
        }
        TypeIr::Named { .. } if type_name_is(ty, DART_VOID) => false,
        TypeIr::Named { .. } if type_name_is(ty, DART_MAP) => false,
        TypeIr::Named { .. } if is_response_body_type(ty) => false,
        TypeIr::Named { .. } => true,
        TypeIr::Builtin { .. } | TypeIr::Dynamic | TypeIr::Unknown => false,
        TypeIr::Function { .. } | TypeIr::Record { .. } => false,
    }
}

/// Returns true for Dart `void`.
pub(super) fn is_void_type(ty: &TypeIr) -> bool {
    ty.is_named(DART_VOID)
}

/// Returns true when a body value can be sent without calling `toJson`.
pub(super) fn uses_direct_body_value(ty: &TypeIr) -> bool {
    match ty {
        TypeIr::Dynamic | TypeIr::Unknown => true,
        TypeIr::Builtin { .. } => true,
        TypeIr::Named { .. } if type_name_is(ty, DART_LIST) || type_name_is(ty, DART_MAP) => true,
        TypeIr::Named { .. } if type_name_is(ty, DART_OBJECT) || type_name_is(ty, DART_VOID) => {
            true
        }
        TypeIr::Named { .. } | TypeIr::Function { .. } | TypeIr::Record { .. } => false,
    }
}
