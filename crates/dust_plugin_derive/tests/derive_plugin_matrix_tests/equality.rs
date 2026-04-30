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
            &["derive_annotation::Eq"],
        )]),
        &SymbolPlan::default(),
    );

    assert!(
        contribution
            .shared_helpers
            .iter()
            .any(|helper| helper.contains("_dustDeepCollectionEquality"))
    );
    assert!(
        contribution
            .shared_helpers
            .iter()
            .any(|helper| helper.contains("_dustUnorderedDeepCollectionEquality"))
    );

    let members = members_for_class(&contribution, "Catalog");
    let eq = members
        .iter()
        .find(|fragment| fragment.contains("bool operator =="))
        .unwrap();
    let hash = members
        .iter()
        .find(|fragment| fragment.contains("int get hashCode"))
        .unwrap();

    assert!(eq.contains("_dustDeepCollectionEquality.equals(other.products, _dustSelf.products)"));
    assert!(eq.contains("_dustDeepCollectionEquality.equals(other.bySku, _dustSelf.bySku)"));
    assert!(eq.contains(
        "_dustUnorderedDeepCollectionEquality.equals(other.featuredSkus, _dustSelf.featuredSkus)"
    ));
    assert!(hash.contains("_dustDeepCollectionEquality.hash(_dustSelf.products)"));
    assert!(hash.contains("_dustDeepCollectionEquality.hash(_dustSelf.bySku)"));
    assert!(hash.contains("_dustUnorderedDeepCollectionEquality.hash(_dustSelf.featuredSkus)"));
}
