use dust_ir::SerdeRenameRuleIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{
    enum_ir, enum_variant, library, renamed_enum, renamed_enum_variant, skipped_enum_variant,
};

#[test]
fn generates_serde_for_enums() {
    let plugin = register_plugin();
    let library = library(
        vec![],
        vec![enum_ir(
            "Status",
            vec![enum_variant("pending"), enum_variant("active")],
            &["dust_dart::Serialize", "dust_dart::Deserialize"],
        )],
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
    assert_eq!(contribution.top_level_functions.len(), 2);

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Object? _$StatusSerialize(Status instance) {
  return switch (instance) {
    Status.pending => 'pending',
    Status.active => 'active',
  };
}

Object? _$StatusToJson(Status instance) =>
    _$StatusSerialize(instance);"#
    );
    assert_eq!(
        from_json,
        r#"Status _$StatusDeserialize(Object? json, [String key = 'json']) {
  return switch (json) {
    'pending' => Status.pending,
    'active' => Status.active,
    _ => throw ArgumentError.value(json, key, 'unknown value for Status at $key'),
  };
}

Status _$StatusFromJson(Object? json, [String key = 'json']) =>
    _$StatusDeserialize(json, key);"#
    );
    assert_eq!(
        contribution.support_types,
        vec![
            r#"final class $StatusSerializer implements Serializer<Status, Object?> {
  const $StatusSerializer();

  @override
  Object? serialize(Status value) => _$StatusSerialize(value);
}"#,
            r#"final class $StatusDeserializer implements Deserializer<Status, Object?> {
  const $StatusDeserializer();

  @override
  Status deserialize(Object? json) => _$StatusDeserialize(json);
}"#
        ]
    );
}

#[test]
fn supports_enum_renaming() {
    let plugin = register_plugin();
    let library = library(
        vec![],
        vec![renamed_enum(
            "UserRole",
            vec![enum_variant("superAdmin"), enum_variant("guestUser")],
            SerdeRenameRuleIr::SnakeCase,
        )],
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

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Object? _$UserRoleSerialize(UserRole instance) {
  return switch (instance) {
    UserRole.superAdmin => 'super_admin',
    UserRole.guestUser => 'guest_user',
  };
}

Object? _$UserRoleToJson(UserRole instance) =>
    _$UserRoleSerialize(instance);"#
    );
    assert_eq!(
        from_json,
        r#"UserRole _$UserRoleDeserialize(Object? json, [String key = 'json']) {
  return switch (json) {
    'super_admin' => UserRole.superAdmin,
    'guest_user' => UserRole.guestUser,
    _ => throw ArgumentError.value(json, key, 'unknown value for UserRole at $key'),
  };
}

UserRole _$UserRoleFromJson(Object? json, [String key = 'json']) =>
    _$UserRoleDeserialize(json, key);"#
    );
}

#[test]
fn supports_enum_variant_rename_and_skip() {
    let plugin = register_plugin();
    let library = library(
        vec![],
        vec![enum_ir(
            "PaymentStatus",
            vec![
                renamed_enum_variant("pendingReview", "pending"),
                enum_variant("paid"),
                skipped_enum_variant("legacyFailed"),
            ],
            &["dust_dart::Serialize", "dust_dart::Deserialize"],
        )],
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
    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Object? _$PaymentStatusSerialize(PaymentStatus instance) {
  return switch (instance) {
    PaymentStatus.pendingReview => 'pending',
    PaymentStatus.paid => 'paid',
    _ => throw ArgumentError.value(instance, 'instance', 'skipped value for PaymentStatus'),
  };
}

Object? _$PaymentStatusToJson(PaymentStatus instance) =>
    _$PaymentStatusSerialize(instance);"#
    );
    assert_eq!(
        from_json,
        r#"PaymentStatus _$PaymentStatusDeserialize(Object? json, [String key = 'json']) {
  return switch (json) {
    'pending' => PaymentStatus.pendingReview,
    'paid' => PaymentStatus.paid,
    _ => throw ArgumentError.value(json, key, 'unknown value for PaymentStatus at $key'),
  };
}

PaymentStatus _$PaymentStatusFromJson(Object? json, [String key = 'json']) =>
    _$PaymentStatusDeserialize(json, key);"#
    );
}
