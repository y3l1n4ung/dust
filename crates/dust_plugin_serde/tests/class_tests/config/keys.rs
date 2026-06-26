use dust_ir::{ParamKind, SerdeClassConfigIr, SerdeFieldConfigIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use crate::support::{class, constructor, constructor_param, field, library};

#[test]
fn generates_disallow_unrecognized_keys_check() {
    let plugin = register_plugin();
    let mut user_class = class(
        "User",
        vec![field("id", TypeIr::string())],
        vec![constructor(
            None,
            vec![constructor_param("id", TypeIr::string(), ParamKind::Named)],
        )],
        &["dust_dart::Deserialize"],
    );
    user_class.serde = Some(SerdeClassConfigIr {
        disallow_unrecognized_keys: true,
        ..Default::default()
    });

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        &contribution.top_level_functions[0],
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  const allowedKeys = <String>{'id'};
  for (final key in json.keys) {
    if (!allowedKeys.contains(key)) {
      throw ArgumentError.value(key, 'json', 'unknown key for User');
    }
  }

  final idValue = JsonHelper.as<String>(json['id'], 'id', 'String');

  return User(id: idValue);
}"#
    );
}

#[test]
fn strict_keys_allow_aliases_but_reject_deserialize_skipped_fields() {
    let plugin = register_plugin();
    let mut user_class = class(
        "User",
        vec![
            dust_ir::FieldIr {
                name: "id".to_owned(),
                ty: TypeIr::string(),
                span: crate::support::span(10, 20),
                has_default: false,
                serde: Some(SerdeFieldConfigIr {
                    aliases: vec!["user_id".to_owned()],
                    ..Default::default()
                }),
                configs: Vec::new(),
            },
            dust_ir::FieldIr {
                name: "token".to_owned(),
                ty: TypeIr::string(),
                span: crate::support::span(21, 30),
                has_default: false,
                serde: Some(SerdeFieldConfigIr {
                    skip_deserializing: true,
                    default_value_source: Some("''".to_owned()),
                    aliases: vec!["legacy_token".to_owned()],
                    ..Default::default()
                }),
                configs: Vec::new(),
            },
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("id", TypeIr::string(), ParamKind::Named),
                constructor_param("token", TypeIr::string(), ParamKind::Named),
            ],
        )],
        &["dust_dart::Deserialize"],
    );
    user_class.serde = Some(SerdeClassConfigIr {
        disallow_unrecognized_keys: true,
        ..Default::default()
    });

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        &contribution.top_level_functions[0],
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  const allowedKeys = <String>{'id', 'user_id'};
  for (final key in json.keys) {
    if (!allowedKeys.contains(key)) {
      throw ArgumentError.value(key, 'json', 'unknown key for User');
    }
  }

  var rawIdKey = 'id';
  Object? rawId;
  if (json.containsKey('id')) {
    rawId = json['id'];
  } else if (json.containsKey('user_id')) {
    rawIdKey = 'user_id';
    rawId = json['user_id'];
  }
  final idValue = JsonHelper.as<String>(rawId, rawIdKey, 'String');
  final tokenValue = '';

  return User(id: idValue, token: tokenValue);
}"#
    );
}
