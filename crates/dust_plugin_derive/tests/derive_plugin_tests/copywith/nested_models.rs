use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use crate::support::{members_for_class, span};

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
                        configs: Vec::new(),
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
                            configs: Vec::new(),
                        },
                        FieldIr {
                            name: "prices".to_owned(),
                            ty: TypeIr::list_of(TypeIr::named("Price")),
                            span: span(22, 23),
                            has_default: false,
                            serde: None,
                            configs: Vec::new(),
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
            query_calls: Vec::new(),
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
