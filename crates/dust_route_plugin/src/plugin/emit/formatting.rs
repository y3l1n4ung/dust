use std::path::Path;

use dust_dart_emit::{DART_DYNAMIC, DART_OBJECT_NULLABLE};
use dust_ir::{LibraryIr, TypeIr};

pub(super) fn package_import_uri(library: &LibraryIr) -> Option<String> {
    let source = Path::new(&library.source_path);
    let relative = source
        .strip_prefix(&library.package_root)
        .ok()
        .and_then(|path| path.strip_prefix("lib").ok())
        .or_else(|| source.strip_prefix("lib").ok())?;
    let path = relative
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    Some(format!("package:{}/{}", library.package_name, path))
}

pub(super) fn dart_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, nullable } => {
            format!("{}{}", kind.as_str(), if *nullable { "?" } else { "" })
        }
        TypeIr::Named { name, nullable, .. } => {
            format!("{name}{}", if *nullable { "?" } else { "" })
        }
        TypeIr::Dynamic => DART_DYNAMIC.to_owned(),
        TypeIr::Function {
            signature,
            nullable,
        } => format!("{signature}{}", if *nullable { "?" } else { "" }),
        TypeIr::Record { shape, nullable } => {
            format!("{shape}{}", if *nullable { "?" } else { "" })
        }
        TypeIr::Unknown => DART_OBJECT_NULLABLE.to_owned(),
    }
}
