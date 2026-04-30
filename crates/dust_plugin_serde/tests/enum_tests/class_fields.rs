use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{
    class, constructor, constructor_param, enum_ir, enum_variant, field, library,
};

#[test]
fn handles_enum_fields_in_classes() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "User",
            vec![field("status", TypeIr::named("Status"))],
            vec![constructor(
                None,
                vec![constructor_param(
                    "status",
                    TypeIr::named("Status"),
                    ParamKind::Named,
                )],
            )],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
        vec![enum_ir(
            "Status",
            vec![enum_variant("active")],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$UserToJson"))
        .unwrap();
    let from_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$UserFromJson"))
        .unwrap();

    assert!(to_json.contains("'status': _$StatusToJson(instance.status),"));
    assert!(from_json.contains("final statusValue = _$StatusFromJson(json['status']);"));
}

#[test]
fn handles_nullable_enum_fields() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "User",
            vec![field("status", TypeIr::named("Status").nullable())],
            vec![constructor(
                None,
                vec![constructor_param(
                    "status",
                    TypeIr::named("Status").nullable(),
                    ParamKind::Named,
                )],
            )],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
        vec![enum_ir(
            "Status",
            vec![enum_variant("active")],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$UserToJson"))
        .unwrap();
    let from_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$UserFromJson"))
        .unwrap();

    assert!(to_json.contains(
        "'status': instance.status == null ? null : _$StatusToJson((instance.status!)),"
    ));
    assert!(from_json.contains(
        "final statusValue = json['status'] == null\n                      ? null\n                      : _$StatusFromJson(json['status']);"
    ));
}

#[test]
fn handles_enums_in_collections() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "Bundle",
            vec![field(
                "roles",
                TypeIr::generic("List", vec![TypeIr::named("Role")]),
            )],
            vec![constructor(
                None,
                vec![constructor_param(
                    "roles",
                    TypeIr::generic("List", vec![TypeIr::named("Role")]),
                    ParamKind::Named,
                )],
            )],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
        vec![enum_ir(
            "Role",
            vec![enum_variant("admin")],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$BundleToJson"))
        .unwrap();
    let from_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$BundleFromJson"))
        .unwrap();

    assert!(
        to_json.contains("'roles': instance.roles.map((item) => _$RoleToJson(item)).toList(),")
    );
    assert!(from_json.contains(
        "final rolesValue = _dustJsonAsList(json['roles'], 'roles').map((item) => _$RoleFromJson(item)).toList();"
    ));
}
