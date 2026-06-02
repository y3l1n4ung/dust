use dust_ir::TypeIr;

pub(super) fn is_supported_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some("String" | "int" | "double" | "num" | "bool" | "DateTime")
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
        TypeIr::Dynamic => "dynamic".to_owned(),
        TypeIr::Unknown => "unknown".to_owned(),
    }
}
