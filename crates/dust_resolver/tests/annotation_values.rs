//! Integration tests for parser-owned annotation values resolved into IR.

use dust_ir::{AnnotationNumberKindIr, AnnotationValueIr};
use dust_parser_dart::{ParseBackend, ParseOptions};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::{SymbolCatalog, resolve_library};
use dust_text::{FileId, SourceText};

#[test]
fn resolves_parser_owned_annotation_values_into_ir() {
    let source = SourceText::new(
        FileId::new(41),
        r#"
part 'user.g.dart';

@SerDe(defaultValue: null, aliases: ['id'], retryable: false, count: 1, ratio: 1.5)
class User {
  const User();
}
"#,
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(41),
        "lib/user.dart",
        "lib/user.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let config = &resolved.library.classes[0].configs[0];
    assert_eq!(
        config.named_argument_value("defaultValue"),
        Some(&AnnotationValueIr::Null)
    );
    assert_eq!(
        config.named_argument_value("aliases"),
        Some(&AnnotationValueIr::List(Vec::new()))
    );
    assert_eq!(
        config.named_argument_value("retryable"),
        Some(&AnnotationValueIr::Bool(false))
    );
    assert_eq!(
        config.named_argument_value("count"),
        Some(&AnnotationValueIr::Number {
            source: "1".to_owned(),
            kind: AnnotationNumberKindIr::Int,
        })
    );
    assert_eq!(
        config.named_argument_value("ratio"),
        Some(&AnnotationValueIr::Number {
            source: "1.5".to_owned(),
            kind: AnnotationNumberKindIr::Double,
        })
    );
}
