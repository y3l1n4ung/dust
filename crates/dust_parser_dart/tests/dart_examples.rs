use dust_parser_dart::{
    ParameterKind, ParsedAnnotation, ParsedClassKind, ParsedClassSurface,
    ParsedConstructorParamSurface, ParsedConstructorSurface, ParsedDirective, ParsedFieldSurface,
    ParsedLibrarySurface,
};
use dust_text::TextRange;

#[test]
fn parse_surface_can_model_real_dart_library_shapes() {
    let library = ParsedLibrarySurface {
        span: TextRange::new(0_u32, 180_u32),
        directives: vec![
            ParsedDirective::Import {
                uri: "dart:convert".to_owned(),
                span: TextRange::new(0_u32, 22_u32),
            },
            ParsedDirective::Part {
                uri: "user.g.dart".to_owned(),
                span: TextRange::new(23_u32, 42_u32),
            },
        ],
        classes: vec![ParsedClassSurface {
            kind: ParsedClassKind::Class,
            name: "User".to_owned(),
            is_abstract: true,
            superclass_name: Some("Entity".to_owned()),
            annotations: vec![ParsedAnnotation {
                name: "Derive".to_owned(),
                arguments_source: Some("[ToString(), Serialize(), Deserialize()]".to_owned()),
                span: TextRange::new(44_u32, 84_u32),
            }],
            fields: vec![
                ParsedFieldSurface {
                    name: "name".to_owned(),
                    annotations: vec![ParsedAnnotation {
                        name: "SerDe".to_owned(),
                        arguments_source: Some("rename: 'full_name'".to_owned()),
                        span: TextRange::new(95_u32, 121_u32),
                    }],
                    type_source: Some("String".to_owned()),
                    has_default: false,
                    span: TextRange::new(100_u32, 118_u32),
                },
                ParsedFieldSurface {
                    name: "age".to_owned(),
                    annotations: Vec::new(),
                    type_source: Some("int?".to_owned()),
                    has_default: false,
                    span: TextRange::new(121_u32, 136_u32),
                },
            ],
            constructors: vec![ParsedConstructorSurface {
                name: Some("named".to_owned()),
                params: vec![
                    ParsedConstructorParamSurface {
                        name: "name".to_owned(),
                        type_source: None,
                        kind: ParameterKind::Named,
                        has_default: false,
                        span: TextRange::new(150_u32, 164_u32),
                    },
                    ParsedConstructorParamSurface {
                        name: "age".to_owned(),
                        type_source: None,
                        kind: ParameterKind::Named,
                        has_default: false,
                        span: TextRange::new(166_u32, 174_u32),
                    },
                ],
                span: TextRange::new(137_u32, 176_u32),
            }],
            span: TextRange::new(85_u32, 179_u32),
        }],
        enums: Vec::new(),
    };

    assert!(!library.is_empty());
    assert_eq!(library.directives.len(), 2);
    assert_eq!(library.classes.len(), 1);
    assert!(library.classes[0].has_annotation("Derive"));
    assert!(library.classes[0].is_abstract);
    assert_eq!(
        library.classes[0].superclass_name.as_deref(),
        Some("Entity")
    );
    assert_eq!(
        library.classes[0].fields[1].type_source.as_deref(),
        Some("int?")
    );
    assert!(library.classes[0].fields[0].has_annotation("SerDe"));
    assert_eq!(
        library.classes[0].constructors[0].params[0].kind,
        ParameterKind::Named
    );
}
