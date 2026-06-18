use dust_ir::{NameIr, ParamKind, TopLevelVariableIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{
    class, constructor, constructor_param, field, library, members_for_class, span,
};

#[test]
fn emits_deep_equality_and_hash_for_collection_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Catalog",
            vec![
                field("products", TypeIr::list_of(TypeIr::named("Product"))),
                field(
                    "bySku",
                    TypeIr::map_of(TypeIr::string(), TypeIr::named("Product")),
                ),
                field(
                    "featuredSkus",
                    TypeIr::generic("Set", vec![TypeIr::string()]),
                ),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param(
                        "products",
                        TypeIr::list_of(TypeIr::named("Product")),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "bySku",
                        TypeIr::map_of(TypeIr::string(), TypeIr::named("Product")),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "featuredSkus",
                        TypeIr::generic("Set", vec![TypeIr::string()]),
                        ParamKind::Positional,
                    ),
                ],
            )],
            &["dust_dart::Eq"],
        )]),
        &SymbolPlan::default(),
    );

    assert_eq!(
        contribution.shared_helpers,
        [
            "const DeepCollectionEquality _catalogProductsEquality = DeepCollectionEquality();"
                .to_owned(),
            "const DeepCollectionEquality _catalogBySkuEquality = DeepCollectionEquality();"
                .to_owned(),
            "const DeepCollectionEquality _catalogFeaturedSkusEquality = DeepCollectionEquality.unordered();"
                .to_owned(),
        ]
    );

    let members = members_for_class(&contribution, "Catalog");
    assert_eq!(
        members,
        [
            r#"@override
bool operator ==(Object other) {
  final self = this as Catalog;
  return identical(this, other) ||
      other is Catalog &&
          runtimeType == other.runtimeType &&
          _catalogProductsEquality.equals(other.products, self.products) &&
          _catalogBySkuEquality.equals(other.bySku, self.bySku) &&
          _catalogFeaturedSkusEquality.equals(other.featuredSkus, self.featuredSkus);
}"#
            .to_owned(),
            r#"@override
int get hashCode {
  final self = this as Catalog;
  return Object.hashAll([
    runtimeType,
    _catalogProductsEquality.hash(self.products),
    _catalogBySkuEquality.hash(self.bySku),
    _catalogFeaturedSkusEquality.hash(self.featuredSkus),
  ]);
}"#
            .to_owned(),
        ]
        .as_slice()
    );
}

#[test]
fn suffixes_deep_equality_helper_when_private_name_collides() {
    let plugin = register_plugin();
    let mut library = library(vec![class(
        "Catalog",
        vec![field("products", TypeIr::list_of(TypeIr::named("Product")))],
        vec![constructor(
            None,
            vec![constructor_param(
                "products",
                TypeIr::list_of(TypeIr::named("Product")),
                ParamKind::Positional,
            )],
        )],
        &["dust_dart::Eq"],
    )]);
    library.variables.push(TopLevelVariableIr {
        name: name("_catalogProductsEquality"),
        ty: TypeIr::named("DeepCollectionEquality"),
        initializer: None,
        annotations: Vec::new(),
        span: span(100, 130),
    });

    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        contribution.shared_helpers,
        [
            "const DeepCollectionEquality _catalogProductsEquality2 = DeepCollectionEquality();"
                .to_owned(),
        ]
    );
    let members = members_for_class(&contribution, "Catalog");
    assert_eq!(
        members,
        [
            r#"@override
bool operator ==(Object other) {
  final self = this as Catalog;
  return identical(this, other) ||
      other is Catalog &&
          runtimeType == other.runtimeType &&
          _catalogProductsEquality2.equals(other.products, self.products);
}"#
            .to_owned(),
            r#"@override
int get hashCode {
  final self = this as Catalog;
  return Object.hashAll([
    runtimeType,
    _catalogProductsEquality2.hash(self.products),
  ]);
}"#
            .to_owned(),
        ]
        .as_slice()
    );
}

fn name(source: &str) -> NameIr {
    NameIr {
        source: source.to_owned(),
        short: source.to_owned(),
        prefix: None,
        span: span(0, source.len() as u32),
    }
}
