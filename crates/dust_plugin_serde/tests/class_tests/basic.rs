use dust_ir::{BuiltinType, ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{
    class, constructor, constructor_param, field, interfaces_for_class, library, members_for_class,
};

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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let members = members_for_class(&contribution, "User");

    assert_eq!(
        interfaces_for_class(&contribution, "User"),
        ["Serializable"]
    );
    assert_eq!(members.len(), 2);
    assert_eq!(
        members[0],
        "Map<String, Object?> serialize() => _$UserSerialize(this as User);"
    );
    assert_eq!(members[1], "Map<String, Object?> toJson() => serialize();");
    assert_eq!(
        contribution.support_types,
        vec![
            r#"final class $UserSerializer implements Serializer<User, Map<String, Object?>> {
  const $UserSerializer();

  @override
  Map<String, Object?> serialize(User value) => _$UserSerialize(value);

  @override
  Map<String, Object?> toJson(User value) => serialize(value);
}"#
        ]
    );
}

#[test]
fn wraps_long_to_json_mixin_member_like_dart_format() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "JsonPaymentSuccess",
            Vec::new(),
            Vec::new(),
            &["dust_dart::Serialize"],
        )],
        vec![],
    );

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let members = members_for_class(&contribution, "JsonPaymentSuccess");

    assert_eq!(members.len(), 2);
    assert_eq!(
        members[0],
        r#"Map<String, Object?> serialize() =>
    _$JsonPaymentSuccessSerialize(this as JsonPaymentSuccess);"#
    );
    assert_eq!(members[1], "Map<String, Object?> toJson() => serialize();");
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let helper = &contribution.top_level_functions[0];

    assert_eq!(
        helper,
        r#"Map<String, Object?> _$UserSerialize(User instance) {
  return <String, Object?>{
    'id': instance.id,
    'age': instance.age,
  };
}

Map<String, Object?> _$UserToJson(User instance) =>
    _$UserSerialize(instance);"#
    );
}

#[test]
fn wraps_from_json_constructor_return_like_dart_format() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "JsonPaymentSuccess",
            vec![
                field("id", TypeIr::string()),
                field("cents", TypeIr::builtin(BuiltinType::Int)),
                field("currency", TypeIr::string()),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param("id", TypeIr::string(), ParamKind::Named),
                    constructor_param("cents", TypeIr::builtin(BuiltinType::Int), ParamKind::Named),
                    constructor_param("currency", TypeIr::string(), ParamKind::Named),
                ],
            )],
            &["dust_dart::Deserialize"],
        )],
        vec![],
    );

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let helper = &contribution.top_level_functions[0];

    assert_eq!(
        helper,
        r#"// factory JsonPaymentSuccess.fromJson(Map<String, Object?> json) => _$JsonPaymentSuccessFromJson(json);
JsonPaymentSuccess _$JsonPaymentSuccessDeserialize(Map<String, Object?> json) {
  final idValue = JsonHelper.as<String>(json['id'], 'id', 'String');
  final centsValue = JsonHelper.as<int>(json['cents'], 'cents', 'int');
  final currencyValue = JsonHelper.as<String>(
    json['currency'],
    'currency',
    'String',
  );

  return JsonPaymentSuccess(
    id: idValue,
    cents: centsValue,
    currency: currencyValue,
  );
}

JsonPaymentSuccess _$JsonPaymentSuccessFromJson(Map<String, Object?> json) =>
    _$JsonPaymentSuccessDeserialize(json);"#
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

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let helper = &contribution.top_level_functions[0];

    assert_eq!(contribution.shared_helpers, Vec::<String>::new());
    assert_eq!(
        helper,
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserDeserialize(Map<String, Object?> json) {
  final idValue = JsonHelper.as<String>(json['id'], 'id', 'String');
  final ageValue = JsonHelper.as<int>(json['age'], 'age', 'int');

  return User(id: idValue, age: ageValue);
}

User _$UserFromJson(Map<String, Object?> json) =>
    _$UserDeserialize(json);"#
    );
    assert_eq!(
        contribution.support_types,
        vec![
            r#"final class $UserDeserializer implements Deserializer<User, Map<String, Object?>> {
  const $UserDeserializer();

  @override
  User deserialize(Map<String, Object?> json) => _$UserDeserialize(json);

  @override
  User fromJson(Map<String, Object?> json) => deserialize(json);
}"#
        ]
    );
}
