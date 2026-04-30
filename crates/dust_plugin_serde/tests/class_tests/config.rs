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

    assert!(to_json.contains("'full_name': instance.fullName,"));
    assert!(from_json.contains(
        "final fullNameValue = _dustJsonAs<String>(json['full_name'], 'full_name', 'String');"
    ));
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

    assert!(from_json.contains("final rawNameKey = json.containsKey('name') ? 'name' : json.containsKey('full_name') ? 'full_name' : json.containsKey('displayName') ? 'displayName' : 'name';"));
    assert!(from_json.contains("final rawName = json.containsKey('name') ? json['name'] : json.containsKey('full_name') ? json['full_name'] : json.containsKey('displayName') ? json['displayName'] : null;"));
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

    assert!(from_json.contains("const allowedKeys = <String>{'id'};"));
    assert!(from_json.contains("for (final key in json.keys) {"));
    assert!(from_json.contains("throw ArgumentError.value(key, 'json', 'unknown key for User');"));
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

    assert!(
        to_json.contains("'createdAt': (const UnixEpochCodec()).serialize(instance.createdAt),")
    );
    assert!(from_json.contains("final createdAtValue = _dustJsonDecodeWithCodec<DateTime>((const UnixEpochCodec()), json['createdAt'], 'createdAt');"));
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

    assert!(from_json.contains(
        "json.containsKey('role') ? _dustJsonAs<String>(json['role'], 'role', 'String') : 'guest'"
    ));
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

    assert!(!to_json.contains("'password'"));
    assert!(to_json.contains("'token': instance.token,"));

    assert!(from_json.contains(
        "final passwordValue = _dustJsonAs<String>(json['password'], 'password', 'String');"
    ));
    assert!(from_json.contains("final tokenValue = '';"));
}
