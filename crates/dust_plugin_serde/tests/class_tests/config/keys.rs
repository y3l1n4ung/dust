use dust_ir::{ParamKind, SerdeClassConfigIr, TypeIr};
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
