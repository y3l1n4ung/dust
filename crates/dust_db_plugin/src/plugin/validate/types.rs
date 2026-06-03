use dust_dart_emit::{
    DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_DYNAMIC, DART_INT, DART_NUM, DART_STRING,
};
use dust_ir::TypeIr;

pub(super) fn is_supported_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some(DART_STRING | DART_INT | DART_DOUBLE | DART_NUM | DART_BOOL | DART_DATE_TIME)
    )
}

pub(super) fn render_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, nullable } => {
            format!("{}{}", kind.as_str(), if *nullable { "?" } else { "" })
        }
        TypeIr::Named { name, nullable, .. } => {
            format!("{name}{}", if *nullable { "?" } else { "" })
        }
        TypeIr::Function {
            signature,
            nullable,
        } => format!("{signature}{}", if *nullable { "?" } else { "" }),
        TypeIr::Record { shape, nullable } => {
            format!("{shape}{}", if *nullable { "?" } else { "" })
        }
        TypeIr::Dynamic => DART_DYNAMIC.to_owned(),
        TypeIr::Unknown => "unknown".to_owned(),
    }
}
