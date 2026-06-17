use dust_diagnostics::Diagnostic;
use dust_parser_dart::{
    AnnotationValue, ParameterKind, ParseBackend, ParseOptions, ParseResult, ParsedAnnotation,
    ParsedClassKind, ParsedClassSurface, ParsedConstructorParamSurface, ParsedConstructorSurface,
    ParsedDirective, ParsedFieldSurface, ParsedLibrarySurface, SourceKind,
    parse_annotation_named_values, parse_file_with_backend,
};
use dust_text::{FileId, SourceText, TextRange};

struct FakeBackend;

impl ParseBackend for FakeBackend {
    fn parse_file(&self, _source: &SourceText, options: ParseOptions) -> ParseResult {
        ParseResult {
            library: ParsedLibrarySurface {
                span: TextRange::new(0_u32, 32_u32),
                directives: vec![ParsedDirective::Part {
                    uri: "user.g.dart".to_owned(),
                    span: TextRange::new(0_u32, 19_u32),
                }],
                classes: vec![ParsedClassSurface {
                    kind: ParsedClassKind::Class,
                    name: "User".to_owned(),
                    is_abstract: false,
                    is_interface: false,
                    superclass_name: Some("Entity".to_owned()),
                    annotations: vec![ParsedAnnotation {
                        name: "Derive".to_owned(),
                        arguments_source: Some("[ToString(), Eq()]".to_owned()),
                        parsed_arguments: None,
                        span: TextRange::new(20_u32, 44_u32),
                    }],
                    fields: vec![ParsedFieldSurface {
                        name: "name".to_owned(),
                        annotations: vec![ParsedAnnotation {
                            name: "SerDe".to_owned(),
                            arguments_source: Some("rename: 'full_name'".to_owned()),
                            parsed_arguments: None,
                            span: TextRange::new(45_u32, 70_u32),
                        }],
                        type_source: Some("String".to_owned()),
                        parsed_type: None,
                        has_default: false,
                        span: TextRange::new(55_u32, 72_u32),
                    }],
                    constructors: vec![ParsedConstructorSurface {
                        name: None,
                        is_factory: false,
                        redirected_target_source: None,
                        redirected_target_name: None,
                        params: vec![ParsedConstructorParamSurface {
                            name: "name".to_owned(),
                            annotations: Vec::new(),
                            type_source: None,
                            parsed_type: None,
                            kind: ParameterKind::Positional,
                            has_default: false,
                            default_value_source: None,
                            span: TextRange::new(80_u32, 89_u32),
                        }],
                        span: TextRange::new(74_u32, 90_u32),
                    }],
                    methods: Vec::new(),
                    span: TextRange::new(45_u32, 90_u32),
                }],
                enums: Vec::new(),
                mixins: Vec::new(),
                extensions: Vec::new(),
                extension_types: Vec::new(),
                functions: Vec::new(),
                variables: Vec::new(),
                typedefs: Vec::new(),
                query_calls: Vec::new(),
            },
            diagnostics: Vec::new(),
            options,
        }
    }
}

#[test]
fn parse_options_default_to_library_mode() {
    let options = ParseOptions::default();

    assert_eq!(options.source_kind, SourceKind::Library);
}

#[test]
fn parse_result_reports_presence_of_diagnostics() {
    let result = ParseResult {
        library: ParsedLibrarySurface {
            span: TextRange::new(0_u32, 0_u32),
            directives: Vec::new(),
            classes: Vec::new(),
            enums: Vec::new(),
            mixins: Vec::new(),
            extensions: Vec::new(),
            extension_types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
            query_calls: Vec::new(),
        },
        diagnostics: vec![Diagnostic::error("unexpected token")],
        options: ParseOptions::default(),
    };

    assert!(result.has_errors());
}

#[test]
fn parse_file_dispatches_through_injected_backend() {
    let source = SourceText::new(FileId::new(1), "class User {}");
    let result = parse_file_with_backend(&FakeBackend, &source, ParseOptions::default());

    assert_eq!(result.library.classes.len(), 1);
    assert_eq!(result.library.classes[0].name, "User");
    assert_eq!(result.library.directives.len(), 1);
    assert!(result.library.classes[0].has_annotation("Derive"));
    assert!(result.library.classes[0].fields[0].has_annotation("SerDe"));
    assert_eq!(
        result.library.classes[0].superclass_name.as_deref(),
        Some("Entity")
    );
}

#[test]
fn directive_span_accessor_returns_stored_span() {
    let directive = ParsedDirective::Import {
        uri: "dart:convert".to_owned(),
        prefix: None,
        span: TextRange::new(0_u32, 22_u32),
    };

    assert_eq!(directive.span(), TextRange::new(0_u32, 22_u32));
}

#[test]
fn parsed_surface_helpers_cover_empty_and_mixin_class_cases() {
    let library = ParsedLibrarySurface {
        span: TextRange::new(0_u32, 0_u32),
        directives: Vec::new(),
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
    let class = ParsedClassSurface {
        kind: ParsedClassKind::MixinClass,
        name: "UserMixin".to_owned(),
        is_abstract: true,
        is_interface: false,
        superclass_name: None,
        annotations: vec![ParsedAnnotation {
            name: "Derive".to_owned(),
            arguments_source: None,
            parsed_arguments: None,
            span: TextRange::new(1_u32, 8_u32),
        }],
        fields: vec![ParsedFieldSurface {
            name: "id".to_owned(),
            annotations: vec![ParsedAnnotation {
                name: "SerDe".to_owned(),
                arguments_source: Some("rename: 'user_id'".to_owned()),
                parsed_arguments: None,
                span: TextRange::new(9_u32, 20_u32),
            }],
            type_source: Some("String".to_owned()),
            parsed_type: None,
            has_default: true,
            span: TextRange::new(21_u32, 35_u32),
        }],
        constructors: Vec::new(),
        methods: Vec::new(),
        span: TextRange::new(0_u32, 40_u32),
    };

    assert!(library.is_empty());
    assert!(class.is_mixin_class());
    assert!(class.has_annotation("Derive"));
    assert!(class.fields[0].has_annotation("SerDe"));
}

#[test]
fn parses_structured_annotation_values() {
    let values = parse_annotation_named_values(
        "(email: true, length: Length(min: 2), tags: ['a', r'b'], custom: User.check)",
    )
    .unwrap();

    assert_eq!(
        values,
        vec![
            ("email".to_owned(), AnnotationValue::Bool(true)),
            (
                "length".to_owned(),
                AnnotationValue::Constructor {
                    name: "Length".to_owned(),
                    named: vec![("min".to_owned(), AnnotationValue::Number("2".to_owned()))],
                },
            ),
            (
                "tags".to_owned(),
                AnnotationValue::List(vec![
                    AnnotationValue::String("a".to_owned()),
                    AnnotationValue::String("b".to_owned()),
                ]),
            ),
            (
                "custom".to_owned(),
                AnnotationValue::Member("User.check".to_owned()),
            ),
        ]
    );
}

#[test]
fn rejects_non_parenthesized_annotation_values() {
    let values = parse_annotation_named_values("email: true");

    assert_eq!(values, None);
}
