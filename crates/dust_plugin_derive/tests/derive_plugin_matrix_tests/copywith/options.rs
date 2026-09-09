//! Option-valued and unknown fields in copyWith.

use super::*;

#[test]
fn copywith_uses_option_update_for_option_and_unknown_fields() {
    let plugin = register_plugin();
    let contribution = plugin
        .generate(
            &library(vec![class(
                "Profile",
                vec![
                    field(
                        "nickname",
                        TypeIr::generic("Option", vec![TypeIr::string()]),
                    ),
                    field("metadata", TypeIr::unknown()),
                ],
                vec![constructor(
                    None,
                    vec![
                        constructor_param(
                            "nickname",
                            TypeIr::generic("Option", vec![TypeIr::string()]),
                            ParamKind::Named,
                        ),
                        constructor_param("metadata", TypeIr::unknown(), ParamKind::Named),
                    ],
                )],
                &["dust_dart::CopyWith"],
            )]),
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    let members = members_for_class(&contribution, "Profile");
    assert_eq!(
        members,
        [r#"/// Creates a copy of this `Profile` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = profile.copyWith();
/// final cleared = profile.copyWith(metadata: null);
/// ```
@pragma('vm:prefer-inline')
_$ProfileCopyWith<Profile> get copyWith => _$ProfileCopyWithImpl<Profile>(this as Profile, (value) => value);"#
        .to_owned()]
        .as_slice()
    );
    assert_eq!(
        contribution.shared_helpers,
        [r#"final class _ProfileCopyWithUnset {
  const _ProfileCopyWithUnset();
}

const _profileCopyWithUnset = _ProfileCopyWithUnset();"#
            .to_owned()]
        .as_slice()
    );
    assert_eq!(
        contribution.support_types,
        [r#"// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$ProfileCopyWith<$Res> {
  $Res call({
    Option<String>? nickname,
    Object? metadata,
  });
}

/// @nodoc
final class _$ProfileCopyWithImpl<$Res> implements _$ProfileCopyWith<$Res> {
  const _$ProfileCopyWithImpl(this._self, this._then);

  final Profile _self;
  final $Res Function(Profile) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? nickname = null,
    Object? metadata = _profileCopyWithUnset,
  }) {
    return _then(
      Profile(
        nickname: nickname == null ? _self.nickname : nickname as Option<String>,
        metadata: identical(metadata, _profileCopyWithUnset)
            ? _self.metadata
            : metadata as Object?,
      )
    );
  }
}"#
        .to_owned()]
        .as_slice()
    );
}
