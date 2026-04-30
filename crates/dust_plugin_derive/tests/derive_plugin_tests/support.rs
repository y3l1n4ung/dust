use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind, SpanIr,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::PluginContribution;
use dust_text::{FileId, TextRange};

pub(crate) fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

pub(crate) fn class_with_traits(name: &str, traits: &[&str]) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(10, 80),
        fields: vec![
            FieldIr {
                name: "id".to_owned(),
                ty: TypeIr::named("String"),
                span: span(20, 30),
                has_default: false,
                serde: None,
            },
            FieldIr {
                name: "age".to_owned(),
                ty: TypeIr::named("int").nullable(),
                span: span(31, 40),
                has_default: false,
                serde: None,
            },
        ],
        constructors: vec![ConstructorIr {
            name: None,
            span: span(40, 60),
            params: vec![
                ConstructorParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::named("String"),
                    span: span(45, 47),
                    kind: ParamKind::Positional,
                    has_default: false,
                },
                ConstructorParamIr {
                    name: "age".to_owned(),
                    ty: TypeIr::named("int").nullable(),
                    span: span(49, 52),
                    kind: ParamKind::Positional,
                    has_default: false,
                },
            ],
        }],
        traits: traits
            .iter()
            .map(|symbol| TraitApplicationIr {
                symbol: SymbolId::new(*symbol),
                span: span(5, 9),
            })
            .collect(),
        serde: None,
    }
}

pub(crate) fn sample_library(traits: &[&str]) -> LibraryIr {
    LibraryIr {
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![class_with_traits("User", traits)],
        enums: Vec::new(),
    }
}

pub(crate) fn members_for_class<'a>(
    contribution: &'a PluginContribution,
    class_name: &str,
) -> &'a [String] {
    contribution
        .mixin_members
        .iter()
        .find(|entry| entry.class_name == class_name)
        .map(|entry| entry.members.as_slice())
        .unwrap_or(&[])
}
