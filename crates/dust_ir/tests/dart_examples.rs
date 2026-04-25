use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind, SpanIr,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_text::{FileId, TextRange};

fn span(file_id: u32, start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(file_id), TextRange::new(start, end))
}

#[test]
fn models_real_dart_data_class_shapes() {
    let library = LibraryIr {
        source_path: "lib/user_profile.dart".to_owned(),
        output_path: "lib/user_profile.g.dart".to_owned(),
        span: span(1, 0, 220),
        classes: vec![ClassIr {
            kind: ClassKindIr::Class,
            name: "UserProfile".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(1, 40, 210),
            fields: vec![
                FieldIr {
                    name: "id".to_owned(),
                    ty: TypeIr::named("String"),
                    span: span(1, 80, 97),
                    has_default: false,
                    serde: None,
                },
                FieldIr {
                    name: "displayName".to_owned(),
                    ty: TypeIr::named("String").nullable(),
                    span: span(1, 100, 126),
                    has_default: false,
                    serde: None,
                },
                FieldIr {
                    name: "tags".to_owned(),
                    ty: TypeIr::generic("List", vec![TypeIr::named("String")]),
                    span: span(1, 129, 154),
                    has_default: false,
                    serde: None,
                },
            ],
            constructors: vec![ConstructorIr {
                name: None,
                span: span(1, 160, 205),
                params: vec![
                    ConstructorParamIr {
                        name: "id".to_owned(),
                        ty: TypeIr::named("String"),
                        span: span(1, 172, 179),
                        kind: ParamKind::Positional,
                        has_default: false,
                    },
                    ConstructorParamIr {
                        name: "displayName".to_owned(),
                        ty: TypeIr::named("String").nullable(),
                        span: span(1, 181, 197),
                        kind: ParamKind::Positional,
                        has_default: false,
                    },
                    ConstructorParamIr {
                        name: "tags".to_owned(),
                        ty: TypeIr::generic("List", vec![TypeIr::named("String")]),
                        span: span(1, 199, 208),
                        kind: ParamKind::Positional,
                        has_default: false,
                    },
                ],
            }],
            traits: vec![
                TraitApplicationIr {
                    symbol: SymbolId::new("derive_annotation::Debug"),
                    span: span(1, 41, 48),
                },
                TraitApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::Serialize"),
                    span: span(1, 50, 61),
                },
                TraitApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::Deserialize"),
                    span: span(1, 63, 76),
                },
            ],
            serde: None,
        }],
    };

    let class = &library.classes[0];
    assert_eq!(class.name, "UserProfile");
    assert_eq!(class.fields.len(), 3);
    assert!(class.fields[1].ty.is_nullable());
    assert_eq!(class.traits.len(), 3);
    assert!(class.constructors[0].can_construct_all_fields(&class.fields));
}
