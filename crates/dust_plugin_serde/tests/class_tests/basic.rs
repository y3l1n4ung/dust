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
            &["dust_dart::Serialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let members = members_for_class(&contribution, "User");

    assert_eq!(members.len(), 1);
    assert_eq!(
        members[0],
        "Map<String, Object?> toJson() => _$UserToJson(this as User);"
    );
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
            &["dust_dart::Serialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let helper = &contribution.top_level_functions[0];

    assert_eq!(
        helper,
        r#"Map<String, Object?> _$UserToJson(User instance) {
  return <String, Object?>{
    'id': instance.id,
    'age': instance.age,
  };
}"#
    );
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
            &["dust_dart::Deserialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let helper = &contribution.top_level_functions[0];

    assert_eq!(
        helper,
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final idValue = _jsonAs<String>(json['id'], 'id', 'String');
  final ageValue = _jsonAs<int>(json['age'], 'age', 'int');

  return User(id: idValue, age: ageValue);
}"#
    );
}
