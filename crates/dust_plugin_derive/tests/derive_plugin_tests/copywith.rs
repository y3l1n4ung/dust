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
            package_root: ".".to_owned(),
            package_name: "dust_test".to_owned(),
            source_path: "lib/catalog.dart".to_owned(),
            output_path: "lib/catalog.g.dart".to_owned(),
            imports: Vec::new(),
            span: span(0, 100),
            classes: vec![ClassIr {
                kind: ClassKindIr::Class,
                name: "Catalog".to_owned(),
                is_abstract: false,
                is_interface: false,
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
                    is_factory: false,
                    redirected_target_source: None,
                    redirected_target_name: None,
                    span: span(40, 60),
                    params: vec![
                        ConstructorParamIr {
                            name: "groups".to_owned(),
                            ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                            span: span(40, 41),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        },
                        ConstructorParamIr {
                            name: "items".to_owned(),
                            ty: TypeIr::list_of(TypeIr::string()),
                            span: span(42, 44),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        },
                        ConstructorParamIr {
                            name: "tags".to_owned(),
                            ty: TypeIr::generic("Set", vec![TypeIr::string()]).nullable(),
                            span: span(45, 47),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        },
                        ConstructorParamIr {
                            name: "metrics".to_owned(),
                            ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                            span: span(48, 49),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        },
                    ],
                }],
                methods: Vec::new(),
                traits: vec![TraitApplicationIr {
                    symbol: SymbolId::new("derive_annotation::CopyWith"),
                    span: span(5, 9),
                }],
                configs: Vec::new(),
                serde: None,
            }],
            enums: Vec::new(),
        },
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Catalog");

    assert_eq!(members.len(), 1);
    assert_eq!(
        members,
        [r#"Catalog copyWith({
  List<List<String>>? groups,
  List<String>? items,
  Object? tags = _undefined,
  Map<String, List<int>>? metrics,
}) {
  final self = this as Catalog;
  final nextGroups = List<List<String>>.of(
    (groups ?? self.groups).map((item_0) => List<String>.of(item_0)),
  );
  final nextItems = List<String>.of(items ?? self.items);
  final nextTagsSource = identical(tags, _undefined)
      ? self.tags
      : tags as Set<String>?;
  final nextTags = nextTagsSource == null ? null : Set<String>.of(nextTagsSource);
  final nextMetrics = Map<String, List<int>>.fromEntries(
    (metrics ?? self.metrics).entries.map(
      (entry_3) => MapEntry(entry_3.key, List<int>.of(entry_3.value)),
    ),
  );

  return Catalog(
    nextGroups,
    nextItems,
    nextTags,
    nextMetrics,
  );
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_clones_nested_dust_models() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            package_root: ".".to_owned(),
            package_name: "dust_test".to_owned(),
            source_path: "lib/product.dart".to_owned(),
            output_path: "lib/product.g.dart".to_owned(),
            imports: Vec::new(),
            span: span(0, 120),
            classes: vec![
                ClassIr {
                    kind: ClassKindIr::Class,
                    name: "Price".to_owned(),
                    is_abstract: false,
                    is_interface: false,
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
                        is_factory: false,
                        redirected_target_source: None,
                        redirected_target_name: None,
                        span: span(3, 4),
                        params: vec![ConstructorParamIr {
                            name: "currency".to_owned(),
                            ty: TypeIr::string(),
                            span: span(3, 4),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        }],
                    }],
                    methods: Vec::new(),
                    traits: vec![TraitApplicationIr {
                        symbol: SymbolId::new("derive_annotation::CopyWith"),
                        span: span(1, 2),
                    }],
                    configs: Vec::new(),
                    serde: None,
                },
                ClassIr {
                    kind: ClassKindIr::Class,
                    name: "Product".to_owned(),
                    is_abstract: false,
                    is_interface: false,
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
                        is_factory: false,
                        redirected_target_source: None,
                        redirected_target_name: None,
                        span: span(24, 25),
                        params: vec![
                            ConstructorParamIr {
                                name: "price".to_owned(),
                                ty: TypeIr::named("Price"),
                                span: span(24, 25),
                                kind: ParamKind::Positional,
                                has_default: false,
                                default_value_source: None,
                            },
                            ConstructorParamIr {
                                name: "prices".to_owned(),
                                ty: TypeIr::list_of(TypeIr::named("Price")),
                                span: span(25, 26),
                                kind: ParamKind::Positional,
                                has_default: false,
                                default_value_source: None,
                            },
                        ],
                    }],
                    methods: Vec::new(),
                    traits: vec![TraitApplicationIr {
                        symbol: SymbolId::new("derive_annotation::CopyWith"),
                        span: span(21, 22),
                    }],
                    configs: Vec::new(),
                    serde: None,
                },
            ],
            enums: Vec::new(),
        },
        &SymbolPlan::default(),
    );

    let members = members_for_class(&contribution, "Product");
    assert_eq!(members.len(), 1);
    assert_eq!(
        members,
        [r#"Product copyWith({
  Price? price,
  List<Price>? prices,
}) {
  final self = this as Product;
  final nextPrice = (price ?? self.price).copyWith();
  final nextPrices = List<Price>.of(
    (prices ?? self.prices).map((item_1) => item_1.copyWith()),
  );

  return Product(
    nextPrice,
    nextPrices,
  );
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_copies_collection_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            package_root: ".".to_owned(),
            package_name: "dust_test".to_owned(),
            source_path: "lib/catalog.dart".to_owned(),
            output_path: "lib/catalog.g.dart".to_owned(),
            imports: Vec::new(),
            span: span(0, 100),
            classes: vec![ClassIr {
                kind: ClassKindIr::Class,
                name: "Catalog".to_owned(),
                is_abstract: false,
                is_interface: false,
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
                    is_factory: false,
                    redirected_target_source: None,
                    redirected_target_name: None,
                    span: span(40, 60),
                    params: vec![
                        ConstructorParamIr {
                            name: "groups".to_owned(),
                            ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                            span: span(40, 41),
                            kind: ParamKind::Named,
                            has_default: false,
                            default_value_source: None,
                        },
                        ConstructorParamIr {
                            name: "metrics".to_owned(),
                            ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                            span: span(48, 49),
                            kind: ParamKind::Named,
                            has_default: false,
                            default_value_source: None,
                        },
                    ],
                }],
                methods: Vec::new(),
                traits: vec![TraitApplicationIr {
                    symbol: SymbolId::new("derive_annotation::CopyWith"),
                    span: span(5, 9),
                }],
                configs: Vec::new(),
                serde: None,
            }],
            enums: Vec::new(),
        },
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Catalog");

    assert_eq!(members.len(), 1);
    assert_eq!(
        members,
        [r#"Catalog copyWith({
  List<List<String>>? groups,
  Map<String, List<int>>? metrics,
}) {
  final self = this as Catalog;
  final nextGroups = List<List<String>>.of(
    (groups ?? self.groups).map((item_0) => List<String>.of(item_0)),
  );
  final nextMetrics = Map<String, List<int>>.fromEntries(
    (metrics ?? self.metrics).entries.map(
      (entry_1) => MapEntry(entry_1.key, List<int>.of(entry_1.value)),
    ),
  );

  return Catalog(
    groups: nextGroups,
    metrics: nextMetrics,
  );
}"#
        .to_owned()]
        .as_slice()
    );
}
