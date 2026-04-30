use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, library};

#[test]
fn handles_nested_serializable_models() {
    let plugin = register_plugin();
    let library = library(
        vec![
            class(
                "User",
                vec![field("profile", TypeIr::named("Profile"))],
                Vec::new(),
                &["derive_serde_annotation::Serialize"],
            ),
            class(
                "Profile",
                Vec::new(),
                Vec::new(),
                &["derive_serde_annotation::Serialize"],
            ),
        ],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let user_to_json = &contribution.top_level_functions[0];

    assert!(user_to_json.contains("'profile': _$ProfileToJson(instance.profile),"));
}

#[test]
fn handles_nested_deserializable_models() {
    let plugin = register_plugin();
    let library = library(
        vec![
            class(
                "User",
                vec![field("profile", TypeIr::named("Profile"))],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "profile",
                        TypeIr::named("Profile"),
                        ParamKind::Named,
                    )],
                )],
                &["derive_serde_annotation::Deserialize"],
            ),
            class(
                "Profile",
                Vec::new(),
                vec![constructor(None, Vec::new())],
                &["derive_serde_annotation::Deserialize"],
            ),
        ],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let user_from_json = &contribution.top_level_functions[0];

    assert!(user_from_json.contains(
        "final profileValue = _$ProfileFromJson(_dustJsonAsMap(json['profile'], 'profile'));"
    ));
}
