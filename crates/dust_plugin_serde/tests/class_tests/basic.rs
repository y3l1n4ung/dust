use dust_ir::{BuiltinType, ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, library, members_for_class};

#[test]
fn generates_to_json_mixin_member() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "User",
            Vec::new(),
            Vec::new(),
            &["derive_serde_annotation::Serialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let members = members_for_class(&contribution, "User");

    assert_eq!(members.len(), 1);
    assert!(members[0].contains("Map<String, Object?> toJson() => _$UserToJson(_dustSelf);"));
}

#[test]
fn generates_to_json_helper() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "User",
            vec![
                field("id", TypeIr::string()),
                field("age", TypeIr::builtin(BuiltinType::Int)),
            ],
            Vec::new(),
            &["derive_serde_annotation::Serialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let helper = &contribution.top_level_functions[0];

    assert!(helper.contains("Map<String, Object?> _$UserToJson(User instance)"));
    assert!(helper.contains("'id': instance.id,"));
    assert!(helper.contains("'age': instance.age,"));
}

#[test]
fn generates_from_json_helper() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "User",
            vec![
                field("id", TypeIr::string()),
                field("age", TypeIr::builtin(BuiltinType::Int)),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param("id", TypeIr::string(), ParamKind::Named),
                    constructor_param("age", TypeIr::builtin(BuiltinType::Int), ParamKind::Named),
                ],
            )],
            &["derive_serde_annotation::Deserialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let helper = &contribution.top_level_functions[0];

    assert!(helper.contains("User _$UserFromJson(Map<String, Object?> json)"));
    assert!(helper.contains("final idValue = _dustJsonAs<String>(json['id'], 'id', 'String');"));
    assert!(helper.contains("final ageValue = _dustJsonAs<int>(json['age'], 'age', 'int');"));
    assert!(helper.contains("return User("));
}
