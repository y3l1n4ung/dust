use std::path::Path;

use dust_dart_emit::{DART_DYNAMIC, DART_OBJECT_NULLABLE};
use dust_ir::{DartFileIr, TypeIr};

/// Builds the package import URI for the source library being generated.
pub(super) fn package_import_uri(library: &DartFileIr) -> Option<String> {
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

/// Renders a Dart type from lowered IR for generated route signatures.
pub(super) fn dart_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, nullable } => {
            format!("{}{}", kind.as_str(), if *nullable { "?" } else { "" })
        }
        TypeIr::Named {
            name,
            args,
            nullable,
        } => {
            let generics = if args.is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    args.iter().map(dart_type).collect::<Vec<_>>().join(", ")
                )
            };
            format!("{name}{generics}{}", if *nullable { "?" } else { "" })
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

#[cfg(test)]
mod tests {
    use dust_ir::DartFileIr;

    use super::*;

    #[test]
    fn package_import_uri_normalizes_lib_paths() {
        assert_eq!(
            package_import_uri(&library(".", "lib/route.dart")),
            Some("package:shop/route.dart".to_owned())
        );
        assert_eq!(
            package_import_uri(&library(
                "examples/shop",
                "examples/shop/lib/router/app_route.dart"
            )),
            Some("package:shop/router/app_route.dart".to_owned())
        );
        assert_eq!(package_import_uri(&library(".", "tool/route.dart")), None);
    }

    #[test]
    fn dart_type_renders_non_named_shapes() {
        assert_eq!(dart_type(&TypeIr::dynamic()), "dynamic");
        assert_eq!(dart_type(&TypeIr::unknown()), "Object?");
        assert_eq!(
            dart_type(&TypeIr::function("void Function(String)").nullable()),
            "void Function(String)?"
        );
        assert_eq!(dart_type(&TypeIr::record("({int id})")), "({int id})");
        assert_eq!(
            dart_type(&TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())).nullable()),
            "Map<String, List<int>>?"
        );
    }

    fn library(package_root: &str, source_path: &str) -> DartFileIr {
        DartFileIr {
            package_root: package_root.to_owned(),
            package_name: "shop".to_owned(),
            source_path: source_path.to_owned(),
            output_path: "lib/route.g.dart".to_owned(),
            imports: Vec::new(),
            library: None,
            library_annotations: Vec::new(),
            import_directives: Vec::new(),
            export_directives: Vec::new(),
            part_directives: Vec::new(),
            part_of: None,
            span: dust_ir::SpanIr::new(
                dust_text::FileId::new(1),
                dust_text::TextRange::new(0_u32, 0_u32),
            ),
            classes: Vec::new(),
            mixins: Vec::new(),
            extensions: Vec::new(),
            extension_types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
            enums: Vec::new(),
            query_calls: Vec::new(),
        }
    }
}
