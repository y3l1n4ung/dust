use dust_ir::{ParamKind, SerdeFieldConfigIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use crate::support::{class, constructor, constructor_param, library, span};

#[test]
fn supports_custom_field_codecs() {
    let plugin = register_plugin();
    let user_class = class(
        "User",
        vec![dust_ir::FieldIr {
            name: "createdAt".to_owned(),
            ty: TypeIr::named("DateTime"),
            span: span(10, 20),
            has_default: false,
            serde: Some(SerdeFieldConfigIr {
                codec_source: Some("const UnixEpochCodec()".to_owned()),
                ..Default::default()
            }),
            configs: Vec::new(),
        }],
        vec![constructor(
            None,
            vec![constructor_param(
                "createdAt",
                TypeIr::named("DateTime"),
                ParamKind::Named,
            )],
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
    'createdAt': (const UnixEpochCodec()).serialize(instance.createdAt),
  };
}"#
    );
    assert_eq!(
        &contribution.top_level_functions[1],
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final createdAtValue = _jsonDecodeWithCodec<DateTime>(
    (const UnixEpochCodec()),
    json['createdAt'],
    'createdAt',
  );

  return User(createdAt: createdAtValue);
}"#
    );
}
