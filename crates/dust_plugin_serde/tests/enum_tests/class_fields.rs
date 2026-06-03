use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{
    class, constructor, constructor_param, enum_ir, enum_variant, field, library,
};

#[test]
fn handles_enum_fields_in_classes() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "User",
            vec![field("status", TypeIr::named("Status"))],
            vec![constructor(
                None,
                vec![constructor_param(
                    "status",
                    TypeIr::named("Status"),
                    ParamKind::Named,
                )],
            )],
            &["dust_dart::Serialize", "dust_dart::Deserialize"],
        )],
        vec![enum_ir(
            "Status",
            vec![enum_variant("active")],
            &["dust_dart::Serialize", "dust_dart::Deserialize"],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Map<String, Object?> _$UserToJson(User instance) {
  return <String, Object?>{
    'status': _$StatusToJson(instance.status),
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final statusValue = _$StatusFromJson(json['status']);

  return User(status: statusValue);
}"#
    );
}

#[test]
fn handles_nullable_enum_fields() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "User",
            vec![field("status", TypeIr::named("Status").nullable())],
            vec![constructor(
                None,
                vec![constructor_param(
                    "status",
                    TypeIr::named("Status").nullable(),
                    ParamKind::Named,
                )],
            )],
            &["dust_dart::Serialize", "dust_dart::Deserialize"],
        )],
        vec![enum_ir(
            "Status",
            vec![enum_variant("active")],
            &["dust_dart::Serialize", "dust_dart::Deserialize"],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Map<String, Object?> _$UserToJson(User instance) {
  return <String, Object?>{
    'status': instance.status == null
        ? null
        : _$StatusToJson((instance.status!)),
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final statusValue = json['status'] == null
      ? null
      : _$StatusFromJson(json['status']);

  return User(status: statusValue);
}"#
    );
}

#[test]
fn handles_enums_in_collections() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "Bundle",
            vec![field(
                "roles",
                TypeIr::generic("List", vec![TypeIr::named("Role")]),
            )],
            vec![constructor(
                None,
                vec![constructor_param(
                    "roles",
                    TypeIr::generic("List", vec![TypeIr::named("Role")]),
                    ParamKind::Named,
                )],
            )],
            &["dust_dart::Serialize", "dust_dart::Deserialize"],
        )],
        vec![enum_ir(
            "Role",
            vec![enum_variant("admin")],
            &["dust_dart::Serialize", "dust_dart::Deserialize"],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Map<String, Object?> _$BundleToJson(Bundle instance) {
  return <String, Object?>{
    'roles': instance.roles
        .map((item) => _$RoleToJson(item))
        .toList(),
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"// factory Bundle.fromJson(Map<String, Object?> json) => _$BundleFromJson(json);
Bundle _$BundleFromJson(Map<String, Object?> json) {
  final rolesValue = _jsonAsList(json['roles'], 'roles')
      .map((item) => _$RoleFromJson(item))
      .toList();

  return Bundle(roles: rolesValue);
}"#
    );
}
