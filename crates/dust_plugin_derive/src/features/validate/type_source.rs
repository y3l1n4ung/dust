use dust_dart_emit::{DART_DOUBLE, DART_INT, DART_LIST, DART_MAP, DART_NUM, DART_SET, DART_STRING};
use dust_ir::TypeIr;

pub(crate) fn render_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, nullable } => nullable_type(kind.as_str(), *nullable),
        TypeIr::Named {
            name,
            args,
            nullable,
        } => {
            let source = if args.is_empty() {
                name.to_string()
            } else {
                let args = args.iter().map(render_type).collect::<Vec<_>>().join(", ");
                format!("{name}<{args}>")
            };
            nullable_type(&source, *nullable)
        }
        TypeIr::Function {
            signature,
            nullable,
        } => nullable_type(signature, *nullable),
        TypeIr::Record { shape, nullable } => nullable_type(shape, *nullable),
        TypeIr::Dynamic => "dynamic".to_owned(),
        TypeIr::Unknown => "Object?".to_owned(),
    }
}

pub(crate) fn input_kind(ty: &TypeIr) -> Option<&'static str> {
    if ty.is_named(DART_STRING) {
        Some("string")
    } else if ty.is_named(DART_INT) {
        Some("int")
    } else if ty.is_named(DART_DOUBLE) {
        Some("double")
    } else if ty.is_named(DART_NUM) {
        Some("num")
    } else {
        None
    }
}

pub(crate) fn supports_length(ty: &TypeIr) -> bool {
    ty.is_named(DART_STRING)
        || ty.is_named(DART_LIST)
        || ty.is_named(DART_SET)
        || ty.is_named(DART_MAP)
}

fn nullable_type(source: &str, nullable: bool) -> String {
    if nullable {
        format!("{source}?")
    } else {
        source.to_owned()
    }
}
