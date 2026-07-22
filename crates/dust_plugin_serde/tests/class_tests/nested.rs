use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, library};

#[test]
fn handles_nested_serializable_models() {
    let plugin = register_plugin();
    let library = library(
        vec![
            class(
                "User",
                vec![field("profile", TypeIr::named("Profile"))],
                Vec::new(),
                &["dust_dart::Serialize"],
            ),
            class("Profile", Vec::new(), Vec::new(), &["dust_dart::Serialize"]),
        ],
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
    let user_to_json = &contribution.top_level_functions[0];

    assert_eq!(
        user_to_json,
        r#"Map<String, Object?> _$UserSerialize(User instance) {
  return <String, Object?>{
    'profile': _$ProfileSerialize(instance.profile),
  };
}

Map<String, Object?> _$UserToJson(User instance) =>
    _$UserSerialize(instance);"#
    );
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
                    vec![constructor_param(
                        "profile",
                        TypeIr::named("Profile"),
                        ParamKind::Named,
                    )],
                )],
                &["dust_dart::Deserialize"],
            ),
            class(
                "Profile",
                Vec::new(),
                vec![constructor(None, Vec::new())],
                &["dust_dart::Deserialize"],
            ),
        ],
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
    let user_from_json = &contribution.top_level_functions[0];

    assert_eq!(
        user_from_json,
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserDeserialize(Map<String, Object?> json) {
  final profileValue = _$ProfileDeserialize(
    JsonHelper.asMap(json['profile'], 'profile'),
  );

  return User(profile: profileValue);
}

User _$UserFromJson(Map<String, Object?> json) =>
    _$UserDeserialize(json);"#
    );
}

#[test]
fn wraps_long_nested_deserializable_model_decode() {
    let plugin = register_plugin();
    let library = library(
        vec![
            class(
                "CheckoutEnvelope",
                vec![field(
                    "deeplyNestedBillingProfile",
                    TypeIr::named("ExtremelyVerboseBillingProfileSnapshot"),
                )],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "deeplyNestedBillingProfile",
                        TypeIr::named("ExtremelyVerboseBillingProfileSnapshot"),
                        ParamKind::Named,
                    )],
                )],
                &["dust_dart::Deserialize"],
            ),
            class(
                "ExtremelyVerboseBillingProfileSnapshot",
                Vec::new(),
                vec![constructor(None, Vec::new())],
                &["dust_dart::Deserialize"],
            ),
        ],
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
    let envelope_from_json = &contribution.top_level_functions[0];

    assert_eq!(
        envelope_from_json,
        r#"// factory CheckoutEnvelope.fromJson(Map<String, Object?> json) => _$CheckoutEnvelopeFromJson(json);
CheckoutEnvelope _$CheckoutEnvelopeDeserialize(Map<String, Object?> json) {
  final deeplyNestedBillingProfileValue = _$ExtremelyVerboseBillingProfileSnapshotDeserialize(
    JsonHelper.asMap(
      json['deeplyNestedBillingProfile'],
      'deeplyNestedBillingProfile',
    ),
  );

  return CheckoutEnvelope(
    deeplyNestedBillingProfile: deeplyNestedBillingProfileValue,
  );
}

CheckoutEnvelope _$CheckoutEnvelopeFromJson(Map<String, Object?> json) =>
    _$CheckoutEnvelopeDeserialize(json);"#
    );
}
