use dust_dart_emit::{
    DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_INT, DART_NUM, DART_STRING, DYNAMIC_TYPES,
};
use dust_ir::TypeIr;

/// Returns true when a type can be fetched through scalar DB helpers.
pub(super) fn is_supported_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some(DART_STRING | DART_INT | DART_DOUBLE | DART_NUM | DART_BOOL | DART_DATE_TIME)
    )
}

/// Renders a type name for validation diagnostics.
pub(super) fn render_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Unknown => "unknown".to_owned(),
        _ => DYNAMIC_TYPES.render(ty),
    }
}
