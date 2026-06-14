#![cfg(test)]

use super::lower_library;
use dust_ir::SpanIr;
use dust_parser_dart::{
    ParsedAnnotation, ParsedAnnotationArgument, ParsedAnnotationArguments,
    ParsedAnnotationNamedArgument, ParsedDirective,
};
use dust_resolver::ResolvedLibrary;
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(42), TextRange::new(start, end))
}

#[test]
fn lowers_parser_directives_into_dart_file_ir() {
    let library = ResolvedLibrary {
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        span: span(0, 160),
        directives: vec![
            ParsedDirective::Library {
                name: Some("models.user".to_owned()),
                annotations: vec![ParsedAnnotation {
                    name: "Deprecated".to_owned(),
                    arguments_source: Some("('legacy')".to_owned()),
                    parsed_arguments: Some(ParsedAnnotationArguments {
                        positional: vec![ParsedAnnotationArgument {
                            source: "'legacy'".to_owned(),
                            span: TextRange::new(12_u32, 20_u32),
                        }],
                        named: vec![ParsedAnnotationNamedArgument {
                            name: "message".to_owned(),
                            source: "message: 'old'".to_owned(),
                            value_source: "'old'".to_owned(),
                            span: TextRange::new(21_u32, 35_u32),
                            value_span: TextRange::new(30_u32, 35_u32),
                        }],
                    }),
                    span: TextRange::new(0_u32, 36_u32),
                }],
                span: TextRange::new(37_u32, 57_u32),
            },
            ParsedDirective::Import {
                uri: "package:app/src/user.dart".to_owned(),
                prefix: Some("user".to_owned()),
                span: TextRange::new(58_u32, 106_u32),
            },
            ParsedDirective::Export {
                uri: "src/public.dart".to_owned(),
                span: TextRange::new(107_u32, 132_u32),
            },
            ParsedDirective::Part {
                uri: "user.g.dart".to_owned(),
                span: TextRange::new(133_u32, 152_u32),
            },
            ParsedDirective::PartOf {
                library_name: Some("models.user".to_owned()),
                uri: None,
                span: TextRange::new(153_u32, 173_u32),
            },
        ],
        part_uri: Some("user.g.dart".to_owned()),
        classes: Vec::new(),
        enums: Vec::new(),
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        query_calls: Vec::new(),
    };

    let outcome = lower_library(&library);

    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    let file = outcome.value;
    assert_eq!(file.imports, ["package:app/src/user.dart"]);
    assert_eq!(
        file.library
            .as_ref()
            .and_then(|library| library.name.as_ref())
            .map(|name| (name.source.as_str(), name.short.as_str())),
        Some(("models.user", "user"))
    );
    assert_eq!(file.library_annotations[0].short_name, "Deprecated");
    assert_eq!(file.library_annotations[0].positional_args.len(), 1);
    assert_eq!(
        file.library_annotations[0]
            .named_args
            .get("message")
            .map(|value| matches!(value, dust_ir::AnnotationValueIr::Expression(_))),
        Some(true)
    );
    assert_eq!(file.import_directives[0].prefix.as_deref(), Some("user"));
    assert_eq!(file.export_directives[0].uri, "src/public.dart");
    assert_eq!(file.part_directives[0].uri, "user.g.dart");
    assert_eq!(
        file.part_of
            .as_ref()
            .and_then(|part_of| part_of.library_name.as_ref())
            .map(|name| name.source.as_str()),
        Some("models.user")
    );
}
