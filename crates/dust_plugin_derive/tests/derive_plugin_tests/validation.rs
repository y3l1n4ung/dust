use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::DustPlugin;
use dust_plugin_derive::register_plugin;

use super::support::span;

#[test]
fn copywith_requires_reconstructible_constructor() {
    let plugin = register_plugin();
    let broken = LibraryIr {
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::Class,
            name: "User".to_owned(),
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
                span: span(40, 50),
                params: vec![ConstructorParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::named("String"),
                    span: span(42, 44),
                    kind: ParamKind::Positional,
                    has_default: false,
                }],
            }],
            traits: vec![TraitApplicationIr {
                symbol: SymbolId::new("derive_annotation::CopyWith"),
                span: span(5, 9),
            }],
            serde: None,
        }],
        enums: Vec::new(),
    };

    let diagnostics = plugin.validate(&broken);

    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .message
            .contains("`CopyWith` requires a constructor that accepts every field")
    );
}

#[test]
fn copywith_rejects_abstract_classes() {
    let plugin = register_plugin();
    let abstract_library = LibraryIr {
        source_path: "lib/entity.dart".to_owned(),
        output_path: "lib/entity.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::Class,
            name: "Entity".to_owned(),
            is_abstract: true,
            superclass_name: None,
            span: span(10, 80),
            fields: vec![FieldIr {
                name: "id".to_owned(),
                ty: TypeIr::string(),
                span: span(20, 30),
                has_default: false,
                serde: None,
            }],
            constructors: vec![ConstructorIr {
                name: None,
                span: span(40, 50),
                params: vec![ConstructorParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::string(),
                    span: span(42, 44),
                    kind: ParamKind::Positional,
                    has_default: false,
                }],
            }],
            traits: vec![TraitApplicationIr {
                symbol: SymbolId::new("derive_annotation::CopyWith"),
                span: span(5, 9),
            }],
            serde: None,
        }],
        enums: Vec::new(),
    };

    let diagnostics = plugin.validate(&abstract_library);

    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .message
            .contains("cannot target abstract class `Entity`")
    );
}

#[test]
fn rejects_mixin_class_targets() {
    let plugin = register_plugin();
    let mixin_class_library = LibraryIr {
        source_path: "lib/mixin_target.dart".to_owned(),
        output_path: "lib/mixin_target.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::MixinClass,
            name: "MixinTarget".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(10, 80),
            fields: vec![FieldIr {
                name: "id".to_owned(),
                ty: TypeIr::string(),
                span: span(20, 30),
                has_default: false,
                serde: None,
            }],
            constructors: vec![ConstructorIr {
                name: None,
                span: span(40, 50),
                params: vec![ConstructorParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::string(),
                    span: span(42, 44),
                    kind: ParamKind::Positional,
                    has_default: false,
                }],
            }],
            traits: vec![TraitApplicationIr {
                symbol: SymbolId::new("derive_annotation::ToString"),
                span: span(5, 9),
            }],
            serde: None,
        }],
        enums: Vec::new(),
    };

    let diagnostics = plugin.validate(&mixin_class_library);

    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .message
            .contains("does not support `mixin class` targets like `MixinTarget`")
    );
}
