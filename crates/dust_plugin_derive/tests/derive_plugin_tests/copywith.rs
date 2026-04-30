use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{members_for_class, span};

#[test]
fn copywith_copies_collection_fields_without_aliasing() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            source_path: "lib/catalog.dart".to_owned(),
            output_path: "lib/catalog.g.dart".to_owned(),
            span: span(0, 100),
            classes: vec![ClassIr {
                kind: ClassKindIr::Class,
                name: "Catalog".to_owned(),
                is_abstract: false,
                superclass_name: None,
                span: span(10, 80),
                fields: vec![
                    FieldIr {
                        name: "groups".to_owned(),
                        ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                        span: span(18, 19),
                        has_default: false,
                        serde: None,
                    },
                    FieldIr {
                        name: "items".to_owned(),
                        ty: TypeIr::list_of(TypeIr::string()),
                        span: span(20, 30),
                        has_default: false,
                        serde: None,
                    },
                    FieldIr {
                        name: "tags".to_owned(),
                        ty: TypeIr::generic("Set", vec![TypeIr::string()]).nullable(),
                        span: span(31, 40),
                        has_default: false,
                        serde: None,
                    },
                    FieldIr {
                        name: "metrics".to_owned(),
                        ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                        span: span(41, 50),
                        has_default: false,
                        serde: None,
                    },
                ],
                constructors: vec![ConstructorIr {
                    name: None,
                    span: span(40, 60),
                    params: vec![
                        ConstructorParamIr {
                            name: "groups".to_owned(),
                            ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                            span: span(40, 41),
                            kind: ParamKind::Positional,
                            has_default: false,
                        },
                        ConstructorParamIr {
                            name: "items".to_owned(),
                            ty: TypeIr::list_of(TypeIr::string()),
                            span: span(42, 44),
                            kind: ParamKind::Positional,
                            has_default: false,
                        },
                        ConstructorParamIr {
                            name: "tags".to_owned(),
                            ty: TypeIr::generic("Set", vec![TypeIr::string()]).nullable(),
                            span: span(45, 47),
                            kind: ParamKind::Positional,
                            has_default: false,
                        },
                        ConstructorParamIr {
                            name: "metrics".to_owned(),
                            ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                            span: span(48, 49),
                            kind: ParamKind::Positional,
                            has_default: false,
                        },
                    ],
                }],
                traits: vec![TraitApplicationIr {
                    symbol: SymbolId::new("derive_annotation::CopyWith"),
                    span: span(5, 9),
                }],
                serde: None,
            }],
            enums: Vec::new(),
        },
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Catalog");

    assert_eq!(members.len(), 1);
    assert!(members[0].contains("Catalog copyWith({"));
    assert!(members[0].contains("List<List<String>>.of(\n"));
    assert!(
        members[0]
            .contains("(groups ?? _dustSelf.groups).map((item_0) => List<String>.of(item_0)),")
    );
    assert!(members[0].contains("List<String>.of(items ?? _dustSelf.items)"));
    assert!(members[0].contains("nextTagsSource == null ? null : Set<String>.of(nextTagsSource)"));
    assert!(members[0].contains("Map<String, List<int>>.fromEntries("));
    assert!(members[0].contains("List<int>.of(entry_"));
    assert!(members[0].contains(".value)"));
}

#[test]
fn copywith_clones_nested_dust_models() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            source_path: "lib/product.dart".to_owned(),
            output_path: "lib/product.g.dart".to_owned(),
            span: span(0, 120),
            classes: vec![
                ClassIr {
                    kind: ClassKindIr::Class,
                    name: "Price".to_owned(),
                    is_abstract: false,
                    superclass_name: None,
                    span: span(1, 20),
                    fields: vec![FieldIr {
                        name: "currency".to_owned(),
                        ty: TypeIr::string(),
                        span: span(2, 3),
                        has_default: false,
                        serde: None,
                    }],
                    constructors: vec![ConstructorIr {
                        name: None,
                        span: span(3, 4),
                        params: vec![ConstructorParamIr {
                            name: "currency".to_owned(),
                            ty: TypeIr::string(),
                            span: span(3, 4),
                            kind: ParamKind::Positional,
                            has_default: false,
                        }],
                    }],
                    traits: vec![TraitApplicationIr {
                        symbol: SymbolId::new("derive_annotation::CopyWith"),
                        span: span(1, 2),
                    }],
                    serde: None,
                },
                ClassIr {
                    kind: ClassKindIr::Class,
                    name: "Product".to_owned(),
                    is_abstract: false,
                    superclass_name: None,
                    span: span(20, 100),
                    fields: vec![
                        FieldIr {
                            name: "price".to_owned(),
                            ty: TypeIr::named("Price"),
                            span: span(21, 22),
                            has_default: false,
                            serde: None,
                        },
                        FieldIr {
                            name: "prices".to_owned(),
                            ty: TypeIr::list_of(TypeIr::named("Price")),
                            span: span(22, 23),
                            has_default: false,
                            serde: None,
                        },
                    ],
                    constructors: vec![ConstructorIr {
                        name: None,
                        span: span(24, 25),
                        params: vec![
                            ConstructorParamIr {
                                name: "price".to_owned(),
                                ty: TypeIr::named("Price"),
                                span: span(24, 25),
                                kind: ParamKind::Positional,
                                has_default: false,
                            },
                            ConstructorParamIr {
                                name: "prices".to_owned(),
                                ty: TypeIr::list_of(TypeIr::named("Price")),
                                span: span(25, 26),
                                kind: ParamKind::Positional,
                                has_default: false,
                            },
                        ],
                    }],
                    traits: vec![TraitApplicationIr {
                        symbol: SymbolId::new("derive_annotation::CopyWith"),
                        span: span(21, 22),
                    }],
                    serde: None,
                },
            ],
            enums: Vec::new(),
        },
        &SymbolPlan::default(),
    );

    let members = members_for_class(&contribution, "Product");
    assert_eq!(members.len(), 1);
    assert!(members.iter().any(|fragment| {
        fragment.contains("Product copyWith({")
            && !fragment.contains("final nextPriceSource = price ?? _dustSelf.price;")
            && fragment.contains("final nextPrice = (price ?? _dustSelf.price).copyWith();")
            && fragment.contains("final nextPrices = List<Price>.of(\n")
            && fragment.contains("(prices ?? _dustSelf.prices).map((")
            && fragment.contains("=> item_")
            && fragment.contains(".copyWith()),")
    }));
}

#[test]
fn copywith_copies_collection_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            source_path: "lib/catalog.dart".to_owned(),
            output_path: "lib/catalog.g.dart".to_owned(),
            span: span(0, 100),
            classes: vec![ClassIr {
                kind: ClassKindIr::Class,
                name: "Catalog".to_owned(),
                is_abstract: false,
                superclass_name: None,
                span: span(10, 80),
                fields: vec![
                    FieldIr {
                        name: "groups".to_owned(),
                        ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                        span: span(18, 19),
                        has_default: false,
                        serde: None,
                    },
                    FieldIr {
                        name: "metrics".to_owned(),
                        ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                        span: span(41, 50),
                        has_default: false,
                        serde: None,
                    },
                ],
                constructors: vec![ConstructorIr {
                    name: None,
                    span: span(40, 60),
                    params: vec![
                        ConstructorParamIr {
                            name: "groups".to_owned(),
                            ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                            span: span(40, 41),
                            kind: ParamKind::Named,
                            has_default: false,
                        },
                        ConstructorParamIr {
                            name: "metrics".to_owned(),
                            ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                            span: span(48, 49),
                            kind: ParamKind::Named,
                            has_default: false,
                        },
                    ],
                }],
                traits: vec![TraitApplicationIr {
                    symbol: SymbolId::new("derive_annotation::CopyWith"),
                    span: span(5, 9),
                }],
                serde: None,
            }],
            enums: Vec::new(),
        },
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Catalog");

    assert_eq!(members.len(), 1);
    assert!(members[0].contains("Catalog copyWith({"));
    assert!(members[0].contains("List<List<String>>? groups,"));
    assert!(!members[0].contains("final nextGroupsSource = groups ?? _dustSelf.groups;"));
    assert!(members[0].contains("final nextGroups = List<List<String>>.of(\n"));
    assert!(
        members[0]
            .contains("(groups ?? _dustSelf.groups).map((item_0) => List<String>.of(item_0)),")
    );
    assert!(members[0].contains("final nextMetrics = Map<String, List<int>>.fromEntries("));
    assert!(members[0].contains("return Catalog("));
}
