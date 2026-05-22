use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{members_for_class, sample_library};

#[test]
fn plugin_claims_core_derive_traits() {
    let plugin = register_plugin();
    let claimed = plugin.claimed_traits();
    assert_eq!(
        claimed,
        vec![
            "derive_annotation::ToString",
            "derive_annotation::Debug",
            "derive_annotation::Eq",
            "derive_annotation::CopyWith",
        ]
    );
}

#[test]
fn eq_requires_no_companion_trait() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&sample_library(&["derive_annotation::Eq"]));

    assert!(diagnostics.is_empty());
}

#[test]
fn requests_undefined_only_for_copywith_when_needed() {
    let plugin = register_plugin();

    let copywith_requested =
        plugin.requested_symbols(&sample_library(&["derive_annotation::CopyWith"]));
    let no_requested = plugin.requested_symbols(&sample_library(&["derive_annotation::ToString"]));

    assert_eq!(copywith_requested, vec!["_undefined".to_owned()]);
    assert!(no_requested.is_empty());
}

#[test]
fn emits_full_fragments_for_matching_traits() {
    let plugin = register_plugin();
    let library = sample_library(&[
        "derive_annotation::ToString",
        "derive_annotation::Eq",
        "derive_annotation::CopyWith",
    ]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let members = members_for_class(&contribution, "User");
    let expected = vec![
        r#"@override
String toString() {
  final self = this as User;
  return 'User('
      'id: ${self.id}, '
      'age: ${self.age}'
      ')';
}"#
        .to_owned(),
        r#"@override
bool operator ==(Object other) {
  final self = this as User;
  return identical(this, other) ||
      other is User &&
          runtimeType == other.runtimeType &&
          other.id == self.id &&
          other.age == self.age;
}"#
        .to_owned(),
        r#"@override
int get hashCode {
  final self = this as User;
  return Object.hashAll([
    runtimeType,
    self.id,
    self.age,
  ]);
}"#
        .to_owned(),
        r#"User copyWith({
  String? id,
  Object? age = _undefined,
}) {
  final self = this as User;
  return User(
    id ?? self.id,
    identical(age, _undefined) ? self.age : age as int?,
  );
}"#
        .to_owned(),
    ];

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(members, expected.as_slice());
}

#[test]
fn legacy_debug_symbol_still_emits_tostring() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &sample_library(&["derive_annotation::Debug"]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "User");

    assert_eq!(
        members,
        [r#"@override
String toString() {
  final self = this as User;
  return 'User('
      'id: ${self.id}, '
      'age: ${self.age}'
      ')';
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn emits_eq_and_hash_fragments_when_eq_is_present() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &sample_library(&["derive_annotation::Eq"]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "User");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(
        members,
        [
            r#"@override
bool operator ==(Object other) {
  final self = this as User;
  return identical(this, other) ||
      other is User &&
          runtimeType == other.runtimeType &&
          other.id == self.id &&
          other.age == self.age;
}"#
            .to_owned(),
            r#"@override
int get hashCode {
  final self = this as User;
  return Object.hashAll([
    runtimeType,
    self.id,
    self.age,
  ]);
}"#
            .to_owned(),
        ]
        .as_slice()
    );
}
