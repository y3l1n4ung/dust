use dust_dart_emit::{DART_DYNAMIC, DART_OBJECT_NULLABLE, OBJECT_NULLABLE_TYPES};
use dust_ir::TypeIr;

/// Renders the public call parameter type for the copyWith interface.
pub(super) fn copy_with_interface_param_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, .. } => nullable_parameter_type(kind.as_str().to_owned()),
        TypeIr::Named { .. } | TypeIr::Function { .. } | TypeIr::Record { .. } => {
            nullable_parameter_type(OBJECT_NULLABLE_TYPES.render(ty))
        }
        TypeIr::Dynamic => DART_DYNAMIC.to_owned(),
        TypeIr::Unknown => DART_OBJECT_NULLABLE.to_owned(),
    }
}

/// Renders the implementation parameter type that can carry the sentinel.
pub(super) fn copy_with_impl_param_type() -> &'static str {
    DART_OBJECT_NULLABLE
}

/// Renders the constructor value expression for one copyWith field.
pub(super) fn copy_with_value_expr(
    field_name: &str,
    ty: &TypeIr,
    self_name: &str,
    sentinel_name: Option<&str>,
) -> String {
    let self_field = format!("{self_name}.{field_name}");
    if needs_copy_with_sentinel(ty) {
        let sentinel = sentinel_name.expect("sentinel field requires sentinel name");
        let replacement = replacement_expr(field_name, ty);
        return format!(
            "identical({field_name}, {sentinel})\n    ? {self_field}\n    : {replacement}"
        );
    }

    let replacement = replacement_expr(field_name, ty);
    format!("{field_name} == null ? {self_field} : {replacement}")
}

/// Returns true when omitted and explicit null need a sentinel.
pub(super) fn needs_copy_with_sentinel(ty: &TypeIr) -> bool {
    ty.is_nullable() || matches!(ty, TypeIr::Dynamic | TypeIr::Unknown)
}

/// Renders a replacement expression with the required cast.
fn replacement_expr(field_name: &str, ty: &TypeIr) -> String {
    if matches!(ty, TypeIr::Dynamic) {
        field_name.to_owned()
    } else {
        format!("{field_name} as {}", replacement_cast_type(ty))
    }
}

/// Renders the cast type used by copyWith replacement expressions.
fn replacement_cast_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, nullable } => nullable_type(kind.as_str().to_owned(), *nullable),
        TypeIr::Named { .. } | TypeIr::Function { .. } | TypeIr::Record { .. } => {
            OBJECT_NULLABLE_TYPES.render(ty)
        }
        TypeIr::Dynamic => DART_DYNAMIC.to_owned(),
        TypeIr::Unknown => DART_OBJECT_NULLABLE.to_owned(),
    }
}

/// Applies nullability to a rendered type.
fn nullable_type(rendered: String, nullable: bool) -> String {
    if nullable {
        nullable_parameter_type(rendered)
    } else {
        rendered
    }
}

/// Ensures a rendered type can accept `null` at the call site.
fn nullable_parameter_type(rendered: String) -> String {
    if rendered.ends_with('?') {
        rendered
    } else {
        format!("{rendered}?")
    }
}
