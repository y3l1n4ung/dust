use dust_ir::{ParamKind, SerdeClassConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use crate::support::{class, constructor, constructor_param, library, span};

#[test]
fn supports_custom_json_key_renaming() {
    let plugin = register_plugin();
    let mut user_class = class(
        "User",
        vec![dust_ir::FieldIr {
            name: "fullName".to_owned(),
            ty: TypeIr::string(),
            span: span(10, 20),
            has_default: false,
            serde: Some(SerdeFieldConfigIr {
                rename: Some("full_name".to_owned()),
                ..Default::default()
            }),
            configs: Vec::new(),
        }],
        vec![constructor(
            None,
            vec![constructor_param(
                "fullName",
                TypeIr::string(),
                ParamKind::Named,
            )],
        )],
        &["dust_dart::Serialize", "dust_dart::Deserialize"],
    );
    user_class.serde = Some(SerdeClassConfigIr {
        rename_all: Some(SerdeRenameRuleIr::KebabCase),
        ..Default::default()
    });

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        &contribution.top_level_functions[0],
        r#"Map<String, Object?> _$UserToJson(User instance) {
  return <String, Object?>{
    'full_name': instance.fullName,
  };
}"#
    );
    assert_eq!(
        &contribution.top_level_functions[1],
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final fullNameValue = JsonHelper.as<String>(
    json['full_name'],
    'full_name',
    'String',
  );

  return User(fullName: fullNameValue);
}"#
    );
}
