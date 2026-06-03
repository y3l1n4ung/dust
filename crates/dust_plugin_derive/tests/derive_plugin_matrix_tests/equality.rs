use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{class, constructor, constructor_param, field, library, members_for_class};

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
            "const DeepCollectionEquality _deepCollectionEquality = DeepCollectionEquality();"
                .to_owned(),
            "const DeepCollectionEquality _unorderedDeepCollectionEquality = DeepCollectionEquality.unordered();"
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
          _deepCollectionEquality.equals(other.products, self.products) &&
          _deepCollectionEquality.equals(other.bySku, self.bySku) &&
          _unorderedDeepCollectionEquality.equals(other.featuredSkus, self.featuredSkus);
}"#
            .to_owned(),
            r#"@override
int get hashCode {
  final self = this as Catalog;
  return Object.hashAll([
    runtimeType,
    _deepCollectionEquality.hash(self.products),
    _deepCollectionEquality.hash(self.bySku),
    _unorderedDeepCollectionEquality.hash(self.featuredSkus),
  ]);
}"#
            .to_owned(),
        ]
        .as_slice()
    );
}
