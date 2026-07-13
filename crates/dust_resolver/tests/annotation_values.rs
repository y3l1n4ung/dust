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

@Meta(
  defaultValue: null,
  aliases: ['id'],
  tags: <String>{'id'},
  weights: {'id': 1},
  labels: (primary: 'id'),
  retryable: false,
  count: 1,
  ratio: 1.5,
  length: Length(2, max: 8)
)
class User {
  const User();
}
"#,
    );
    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("Meta", "test::Meta");

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
        Some(&AnnotationValueIr::List(vec![AnnotationValueIr::String(
            "id".to_owned(),
        )]))
    );
    assert_eq!(
        config.named_argument_value("tags"),
        Some(&AnnotationValueIr::Set(vec![AnnotationValueIr::String(
            "id".to_owned(),
        )]))
    );
    assert_eq!(
        config.named_argument_value("weights"),
        Some(&AnnotationValueIr::Map(vec![(
            AnnotationValueIr::String("id".to_owned()),
            AnnotationValueIr::Number {
                source: "1".to_owned(),
                kind: AnnotationNumberKindIr::Int,
            },
        )]))
    );
    assert_eq!(
        config.named_argument_value("labels"),
        Some(&AnnotationValueIr::Record(vec![(
            "primary".to_owned(),
            AnnotationValueIr::String("id".to_owned())
        ),]))
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
    let Some(AnnotationValueIr::Constructor {
        name,
        positional_args,
        named_args,
    }) = config.named_argument_value("length")
    else {
        panic!("expected structured constructor");
    };
    assert_eq!(name.short, "Length");
    assert_eq!(
        positional_args,
        &[AnnotationValueIr::Number {
            source: "2".to_owned(),
            kind: AnnotationNumberKindIr::Int,
        }]
    );
    assert_eq!(
        named_args.get("max"),
        Some(&AnnotationValueIr::Number {
            source: "8".to_owned(),
            kind: AnnotationNumberKindIr::Int,
        })
    );
}
