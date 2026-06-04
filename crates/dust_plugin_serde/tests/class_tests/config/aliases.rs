use dust_ir::{ParamKind, SerdeFieldConfigIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use crate::support::{class, constructor, constructor_param, library, span};

#[test]
fn supports_field_aliases_during_deserialization() {
    let plugin = register_plugin();
    let user_class = class(
        "User",
        vec![dust_ir::FieldIr {
            name: "name".to_owned(),
            ty: TypeIr::string(),
            span: span(10, 20),
            has_default: false,
            serde: Some(SerdeFieldConfigIr {
                aliases: vec!["full_name".to_owned(), "displayName".to_owned()],
                ..Default::default()
            }),
            configs: Vec::new(),
        }],
        vec![constructor(
            None,
            vec![constructor_param(
                "name",
                TypeIr::string(),
                ParamKind::Named,
            )],
        )],
        &["dust_dart::Deserialize"],
    );

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        &contribution.top_level_functions[0],
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  var rawNameKey = 'name';
  Object? rawName;
  if (json.containsKey('name')) {
    rawName = json['name'];
  } else if (json.containsKey('full_name')) {
    rawNameKey = 'full_name';
    rawName = json['full_name'];
  } else if (json.containsKey('displayName')) {
    rawNameKey = 'displayName';
    rawName = json['displayName'];
  }
  final nameValue = JsonHelper.as<String>(rawName, rawNameKey, 'String');

  return User(name: nameValue);
}"#
    );
}
