use dust_ir::{ParamKind, SerdeClassConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, library, span};

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
        }],
        vec![constructor(
            None,
            vec![constructor_param(
                "fullName",
                TypeIr::string(),
                ParamKind::Named,
            )],
        )],
        &[
            "derive_serde_annotation::Serialize",
            "derive_serde_annotation::Deserialize",
        ],
    );
    user_class.serde = Some(SerdeClassConfigIr {
        rename_all: Some(SerdeRenameRuleIr::KebabCase),
        ..Default::default()
    });

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Map<String, Object?> _$UserToJson(User instance) {
  return <String, Object?>{
    'full_name': instance.fullName,
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final fullNameValue = _jsonAs<String>(
    json['full_name'],
    'full_name',
    'String',
  );

  return User(fullName: fullNameValue);
}"#
    );
}

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
        }],
        vec![constructor(
            None,
            vec![constructor_param(
                "name",
                TypeIr::string(),
                ParamKind::Named,
            )],
        )],
        &["derive_serde_annotation::Deserialize"],
    );

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let from_json = &contribution.top_level_functions[0];

    assert_eq!(
        from_json,
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
  final nameValue = _jsonAs<String>(rawName, rawNameKey, 'String');

  return User(name: nameValue);
}"#
    );
}

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
        &["derive_serde_annotation::Deserialize"],
    );
    user_class.serde = Some(SerdeClassConfigIr {
        disallow_unrecognized_keys: true,
        ..Default::default()
    });

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let from_json = &contribution.top_level_functions[0];

    assert_eq!(
        from_json,
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  const allowedKeys = <String>{'id'};
  for (final key in json.keys) {
    if (!allowedKeys.contains(key)) {
      throw ArgumentError.value(key, 'json', 'unknown key for User');
    }
  }

  final idValue = _jsonAs<String>(json['id'], 'id', 'String');

  return User(id: idValue);
}"#
    );
}

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

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Map<String, Object?> _$UserToJson(User instance) {
  return <String, Object?>{
    'createdAt': (const UnixEpochCodec()).serialize(instance.createdAt),
  };
}"#
    );
    assert_eq!(
        from_json,
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

#[test]
fn supports_default_values_during_deserialization() {
    let plugin = register_plugin();
    let user_class = class(
        "User",
        vec![dust_ir::FieldIr {
            name: "role".to_owned(),
            ty: TypeIr::string(),
            span: span(10, 20),
            has_default: false,
            serde: Some(SerdeFieldConfigIr {
                default_value_source: Some("'guest'".to_owned()),
                ..Default::default()
            }),
        }],
        vec![constructor(
            None,
            vec![constructor_param(
                "role",
                TypeIr::string(),
                ParamKind::Named,
            )],
        )],
        &["derive_serde_annotation::Deserialize"],
    );

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let from_json = &contribution.top_level_functions[0];

    assert_eq!(
        from_json,
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final roleValue = json.containsKey('role')
      ? _jsonAs<String>(json['role'], 'role', 'String')
      : 'guest';

  return User(role: roleValue);
}"#
    );
}

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

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Map<String, Object?> _$UserToJson(User instance) {
  return <String, Object?>{
    'token': instance.token,
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final passwordValue = _jsonAs<String>(json['password'], 'password', 'String');
  final tokenValue = '';

  return User(password: passwordValue, token: tokenValue);
}"#
    );
}
