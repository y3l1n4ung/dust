use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, EnumIr, FieldIr, LibraryIr, ParamKind,
    SerdeClassConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr, SpanIr, SymbolId,
    TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(11), TextRange::new(start, end))
}

fn trait_application(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
        span: span(1, 5),
    }
}

fn field(name: &str, ty: TypeIr) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(10, 20),
        has_default: false,
        serde: None,
    }
}

fn constructor_param(name: &str, ty: TypeIr, kind: ParamKind) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(30, 35),
        kind,
        has_default: false,
    }
}

fn constructor(name: Option<&str>, params: Vec<ConstructorParamIr>) -> ConstructorIr {
    ConstructorIr {
        name: name.map(str::to_owned),
        span: span(25, 60),
        params,
    }
}

fn class(
    name: &str,
    fields: Vec<FieldIr>,
    constructors: Vec<ConstructorIr>,
    traits: &[&str],
) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(0, 100),
        fields,
        constructors,
        traits: traits
            .iter()
            .map(|symbol| trait_application(symbol))
            .collect(),
        serde: None,
    }
}

fn library(classes: Vec<ClassIr>, enums: Vec<EnumIr>) -> LibraryIr {
    LibraryIr {
        source_path: "lib/models.dart".to_owned(),
        output_path: "lib/models.g.dart".to_owned(),
        span: span(0, 200),
        classes,
        enums,
    }
}

fn members_for_class<'a>(
    contribution: &'a dust_plugin_api::PluginContribution,
    class_name: &str,
) -> &'a [String] {
    contribution
        .mixin_members
        .iter()
        .find(|entry| entry.class_name == class_name)
        .map(|entry| entry.members.as_slice())
        .unwrap_or(&[])
}

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
                field("age", TypeIr::builtin(dust_ir::BuiltinType::Int)),
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
                field("age", TypeIr::builtin(dust_ir::BuiltinType::Int)),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param("id", TypeIr::string(), ParamKind::Named),
                    constructor_param(
                        "age",
                        TypeIr::builtin(dust_ir::BuiltinType::Int),
                        ParamKind::Named,
                    ),
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

#[test]
fn handles_nested_serializable_models() {
    let plugin = register_plugin();
    let library = library(
        vec![
            class(
                "User",
                vec![field("profile", TypeIr::named("Profile"))],
                Vec::new(),
                &["derive_serde_annotation::Serialize"],
            ),
            class(
                "Profile",
                Vec::new(),
                Vec::new(),
                &["derive_serde_annotation::Serialize"],
            ),
        ],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let user_to_json = &contribution.top_level_functions[0];

    assert!(user_to_json.contains("'profile': _$ProfileToJson(instance.profile),"));
}

#[test]
fn handles_nested_deserializable_models() {
    let plugin = register_plugin();
    let library = library(
        vec![
            class(
                "User",
                vec![field("profile", TypeIr::named("Profile"))],
                vec![constructor(
                    None,
                    vec![constructor_param("profile", TypeIr::named("Profile"), ParamKind::Named)],
                )],
                &["derive_serde_annotation::Deserialize"],
            ),
            class(
                "Profile",
                Vec::new(),
                vec![constructor(None, Vec::new())],
                &["derive_serde_annotation::Deserialize"],
            ),
        ],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let user_from_json = &contribution.top_level_functions[0];

    assert!(user_from_json.contains("final profileValue = _$ProfileFromJson(_dustJsonAsMap(json['profile'], 'profile'));"));
}

#[test]
fn supports_custom_json_key_renaming() {
    let plugin = register_plugin();
    let mut user_class = class(
        "User",
        vec![FieldIr {
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
            vec![constructor_param("fullName", TypeIr::string(), ParamKind::Named)],
        )],
        &["derive_serde_annotation::Serialize", "derive_serde_annotation::Deserialize"],
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
    assert!(from_json.contains("final fullNameValue = _dustJsonAs<String>(json['full_name'], 'full_name', 'String');"));
}

#[test]
fn supports_field_aliases_during_deserialization() {
    let plugin = register_plugin();
    let user_class = class(
        "User",
        vec![FieldIr {
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
            vec![constructor_param("name", TypeIr::string(), ParamKind::Named)],
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
        vec![FieldIr {
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
            vec![constructor_param("createdAt", TypeIr::named("DateTime"), ParamKind::Named)],
        )],
        &["derive_serde_annotation::Serialize", "derive_serde_annotation::Deserialize"],
    );

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert!(to_json.contains("'createdAt': (const UnixEpochCodec()).serialize(instance.createdAt),"));
    assert!(from_json.contains("final createdAtValue = _dustJsonDecodeWithCodec<DateTime>((const UnixEpochCodec()), json['createdAt'], 'createdAt');"));
}

#[test]
fn supports_default_values_during_deserialization() {
    let plugin = register_plugin();
    let user_class = class(
        "User",
        vec![FieldIr {
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
            vec![constructor_param("role", TypeIr::string(), ParamKind::Named)],
        )],
        &["derive_serde_annotation::Deserialize"],
    );

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let from_json = &contribution.top_level_functions[0];

    assert!(from_json.contains("json.containsKey('role') ? _dustJsonAs<String>(json['role'], 'role', 'String') : 'guest'"));
}

#[test]
fn supports_skipping_fields() {
    let plugin = register_plugin();
    let user_class = class(
        "User",
        vec![
            FieldIr {
                name: "password".to_owned(),
                ty: TypeIr::string(),
                span: span(10, 20),
                has_default: false,
                serde: Some(SerdeFieldConfigIr {
                    skip_serializing: true,
                    ..Default::default()
                }),
            },
            FieldIr {
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
        &["derive_serde_annotation::Serialize", "derive_serde_annotation::Deserialize"],
    );

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert!(!to_json.contains("'password'"));
    assert!(to_json.contains("'token': instance.token,"));

    assert!(from_json.contains("final passwordValue = _dustJsonAs<String>(json['password'], 'password', 'String');"));
    assert!(from_json.contains("final tokenValue = '';"));
}
