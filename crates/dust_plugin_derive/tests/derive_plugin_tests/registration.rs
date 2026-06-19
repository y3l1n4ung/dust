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
            "dust_dart::ToString",
            "dust_dart::Debug",
            "dust_dart::Eq",
            "dust_dart::CopyWith",
            "dust_dart::Validate",
        ]
    );
    assert_eq!(plugin.claimed_configs(), vec!["dust_dart::Validate"]);
    assert_eq!(
        plugin.supported_annotations(),
        vec!["Derive", "ToString", "Debug", "Eq", "CopyWith", "Validate",]
    );
}

#[test]
fn eq_requires_no_companion_trait() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&sample_library(&["dust_dart::Eq"]));

    assert!(diagnostics.is_empty());
}

#[test]
fn copywith_requires_no_reserved_helpers() {
    let plugin = register_plugin();

    let copywith_requested = plugin.requested_symbols(&sample_library(&["dust_dart::CopyWith"]));
    let no_requested = plugin.requested_symbols(&sample_library(&["dust_dart::ToString"]));

    assert!(copywith_requested.is_empty());
    assert!(no_requested.is_empty());
}

#[test]
fn emits_full_fragments_for_matching_traits() {
    let plugin = register_plugin();
    let library = sample_library(&[
        "dust_dart::ToString",
        "dust_dart::Eq",
        "dust_dart::CopyWith",
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
        r#"/// Creates a copy of this `User` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = user.copyWith(id: 'John');
/// final cleared = user.copyWith(age: null);
/// ```
@pragma('vm:prefer-inline')
_$UserCopyWith<User> get copyWith => _$UserCopyWithImpl<User>(this as User, (value) => value);"#
            .to_owned(),
    ];

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(members, expected.as_slice());
    assert_eq!(
        contribution.shared_helpers,
        [r#"final class _UserCopyWithUnset {
  const _UserCopyWithUnset();
}

const _userCopyWithUnset = _UserCopyWithUnset();"#
            .to_owned()]
        .as_slice()
    );
    assert_eq!(
        contribution.support_types,
        [r#"// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$UserCopyWith<$Res> {
  $Res call({
    String? id,
    int? age,
  });
}

/// @nodoc
final class _$UserCopyWithImpl<$Res> implements _$UserCopyWith<$Res> {
  const _$UserCopyWithImpl(this._self, this._then);

  final User _self;
  final $Res Function(User) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
    Object? age = _userCopyWithUnset,
  }) {
    return _then(
      User(
        id == null ? _self.id : id as String,
        identical(age, _userCopyWithUnset)
            ? _self.age
            : age as int?,
      )
    );
  }
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn legacy_debug_symbol_still_emits_tostring() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &sample_library(&["dust_dart::Debug"]),
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
    let contribution = plugin.emit(&sample_library(&["dust_dart::Eq"]), &SymbolPlan::default());
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
