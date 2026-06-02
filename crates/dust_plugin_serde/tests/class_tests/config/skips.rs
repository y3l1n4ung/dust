use dust_ir::{ParamKind, SerdeFieldConfigIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use crate::support::{class, constructor, constructor_param, library, span};

#[test]
fn supports_skipping_fields() {
    let plugin = register_plugin();
    let user_class = class(
        "User",
        vec![
            dust_ir::FieldIr {
                name: "password".to_owned(),
                ty: TypeIr::string(),
                span: span(10, 20),
                has_default: false,
                serde: Some(SerdeFieldConfigIr {
                    skip_serializing: true,
                    ..Default::default()
                }),
                configs: Vec::new(),
            },
            dust_ir::FieldIr {
                name: "token".to_owned(),
                ty: TypeIr::string(),
                span: span(10, 20),
                has_default: false,
                serde: Some(SerdeFieldConfigIr {
                    skip_deserializing: true,
                    default_value_source: Some("''".to_owned()),
                    ..Default::default()
                }),
                configs: Vec::new(),
            },
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("password", TypeIr::string(), ParamKind::Named),
                constructor_param("token", TypeIr::string(), ParamKind::Named),
            ],
        )],
        &[
            "derive_serde_annotation::Serialize",
            "derive_serde_annotation::Deserialize",
        ],
    );

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        &contribution.top_level_functions[0],
        r#"Map<String, Object?> _$UserToJson(User instance) {
  return <String, Object?>{
    'token': instance.token,
  };
}"#
    );
    assert_eq!(
        &contribution.top_level_functions[1],
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final passwordValue = _jsonAs<String>(json['password'], 'password', 'String');
  final tokenValue = '';

  return User(password: passwordValue, token: tokenValue);
}"#
    );
}
