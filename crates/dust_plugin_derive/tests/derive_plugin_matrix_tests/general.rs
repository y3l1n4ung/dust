use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{class, constructor, constructor_param, field, library, members_for_class};
use dust_ir::{ParamKind, TypeIr};

#[test]
fn emits_debug_eq_and_hash_for_zero_field_class() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Unit",
            Vec::new(),
            vec![constructor(None, Vec::new())],
            &["dust_dart::ToString", "dust_dart::Eq"],
        )]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Unit");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(
        members,
        [
            r#"@override
String toString() {
  return 'Unit()';
}"#
            .to_owned(),
            r#"@override
bool operator ==(Object other) =>
    identical(this, other) ||
    other is Unit &&
        runtimeType == other.runtimeType;"#
                .to_owned(),
            r#"@override
int get hashCode {
  return Object.hashAll([
    runtimeType,
  ]);
}"#
            .to_owned(),
        ]
        .as_slice()
    );
}

#[test]
fn validation_accumulates_multiple_class_errors() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library(vec![class(
        "BrokenCopyWith",
        vec![field("id", TypeIr::string()), field("age", TypeIr::int())],
        vec![constructor(
            None,
            vec![constructor_param(
                "id",
                TypeIr::string(),
                ParamKind::Positional,
            )],
        )],
        &["dust_dart::CopyWith"],
    )]));

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains(
            "`CopyWith` requires a constructor that accepts every field on class `BrokenCopyWith`",
        )
    }));
}

#[test]
fn requested_symbols_are_deduped_for_multiple_copywith_classes() {
    let plugin = register_plugin();
    let requested = plugin.requested_symbols(&library(vec![
        class(
            "User",
            vec![field("id", TypeIr::string())],
            vec![constructor(
                None,
                vec![constructor_param(
                    "id",
                    TypeIr::string(),
                    ParamKind::Positional,
                )],
            )],
            &["dust_dart::CopyWith"],
        ),
        class(
            "Team",
            vec![field("name", TypeIr::string())],
            vec![constructor(
                None,
                vec![constructor_param(
                    "name",
                    TypeIr::string(),
                    ParamKind::Positional,
                )],
            )],
            &["dust_dart::CopyWith"],
        ),
    ]));

    assert!(requested.is_empty());
}

#[test]
fn emits_fragments_for_multiple_classes_in_stable_feature_order() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![
            class(
                "User",
                vec![field("id", TypeIr::string())],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "id",
                        TypeIr::string(),
                        ParamKind::Positional,
                    )],
                )],
                &["dust_dart::ToString", "dust_dart::Eq"],
            ),
            class(
                "Team",
                vec![field("name", TypeIr::string())],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "name",
                        TypeIr::string(),
                        ParamKind::Positional,
                    )],
                )],
                &["dust_dart::CopyWith"],
            ),
        ]),
        &SymbolPlan::default(),
    );
    let user_members = members_for_class(&contribution, "User");
    let team_members = members_for_class(&contribution, "Team");

    assert_eq!(contribution.mixin_members.len(), 2);
    assert_eq!(
        user_members,
        [
            r#"@override
String toString() {
  final self = this as User;
  return 'User('
      'id: ${self.id}'
      ')';
}"#
            .to_owned(),
            r#"@override
bool operator ==(Object other) {
  final self = this as User;
  return identical(this, other) ||
      other is User &&
          runtimeType == other.runtimeType &&
          other.id == self.id;
}"#
            .to_owned(),
            r#"@override
int get hashCode {
  final self = this as User;
  return Object.hashAll([
    runtimeType,
    self.id,
  ]);
}"#
            .to_owned(),
        ]
        .as_slice()
    );
    assert_eq!(
        team_members,
        [
            r#"/// Creates a copy of this `Team` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = team.copyWith(name: 'John');
/// ```
@pragma('vm:prefer-inline')
_$TeamCopyWith<Team> get copyWith => _$TeamCopyWithImpl<Team>(this as Team, (value) => value);"#
                .to_owned()
        ]
        .as_slice()
    );
    assert_eq!(
        contribution.support_types,
        [r#"// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$TeamCopyWith<$Res> {
  $Res call({
    String? name,
  });
}

/// @nodoc
final class _$TeamCopyWithImpl<$Res> implements _$TeamCopyWith<$Res> {
  const _$TeamCopyWithImpl(this._self, this._then);

  final Team _self;
  final $Res Function(Team) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? name = null,
  }) {
    return _then(
      Team(
        name == null ? _self.name : name as String,
      )
    );
  }
}"#
        .to_owned()]
        .as_slice()
    );
}
