use dust_ir::SerdeRenameRuleIr;
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{enum_ir, enum_variant, library, renamed_enum};

#[test]
fn generates_serde_for_enums() {
    let plugin = register_plugin();
    let library = library(
        vec![],
        vec![enum_ir(
            "Status",
            vec![enum_variant("pending"), enum_variant("active")],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    assert_eq!(contribution.top_level_functions.len(), 2);

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Object? _$StatusToJson(Status instance) {
  return switch (instance) {
    Status.pending => 'pending',
    Status.active => 'active',
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"Status _$StatusFromJson(Object? json) {
  return switch (json) {
    'pending' => Status.pending,
    'active' => Status.active,
    _ => throw ArgumentError.value(json, 'json', 'unknown value for Status'),
  };
}"#
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
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Object? _$UserRoleToJson(UserRole instance) {
  return switch (instance) {
    UserRole.superAdmin => 'super_admin',
    UserRole.guestUser => 'guest_user',
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"UserRole _$UserRoleFromJson(Object? json) {
  return switch (json) {
    'super_admin' => UserRole.superAdmin,
    'guest_user' => UserRole.guestUser,
    _ => throw ArgumentError.value(json, 'json', 'unknown value for UserRole'),
  };
}"#
    );
}
