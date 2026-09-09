//! Nullable nested model helpers and named model collections.

use super::*;

#[test]
fn copywith_emits_nullable_nested_model_helper() {
    let plugin = register_plugin();
    let contribution = plugin
        .generate(
            &library(vec![
                class(
                    "Address",
                    vec![field("city", TypeIr::string())],
                    vec![constructor(
                        None,
                        vec![constructor_param(
                            "city",
                            TypeIr::string(),
                            ParamKind::Named,
                        )],
                    )],
                    &["dust_dart::CopyWith"],
                ),
                class(
                    "Profile",
                    vec![
                        field("name", TypeIr::string()),
                        field("nickname", TypeIr::string().nullable()),
                        field("address", TypeIr::named("Address")),
                        field("mailingAddress", TypeIr::named("Address").nullable()),
                    ],
                    vec![constructor(
                        None,
                        vec![
                            constructor_param("name", TypeIr::string(), ParamKind::Named),
                            constructor_param(
                                "nickname",
                                TypeIr::string().nullable(),
                                ParamKind::Named,
                            ),
                            constructor_param(
                                "address",
                                TypeIr::named("Address"),
                                ParamKind::Named,
                            ),
                            constructor_param(
                                "mailingAddress",
                                TypeIr::named("Address").nullable(),
                                ParamKind::Named,
                            ),
                        ],
                    )],
                    &["dust_dart::CopyWith"],
                ),
            ]),
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
/// final updated = profile.copyWith(name: 'John');
/// final cleared = profile.copyWith(nickname: null);
/// final nested = profile.copyWith.address(city: 'London');
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
        [
            r#"// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$AddressCopyWith<$Res> {
  $Res call({
    String? city,
  });
}

/// @nodoc
final class _$AddressCopyWithImpl<$Res> implements _$AddressCopyWith<$Res> {
  const _$AddressCopyWithImpl(this._self, this._then);

  final Address _self;
  final $Res Function(Address) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? city = null,
  }) {
    return _then(
      Address(
        city: city == null ? _self.city : city as String,
      )
    );
  }
}"#
            .to_owned(),
            r#"/// @nodoc
abstract class _$ProfileCopyWith<$Res> {
  $Res call({
    String? name,
    String? nickname,
    Address? address,
    Address? mailingAddress,
  });

  _$AddressCopyWith<$Res> get address;

  _$AddressCopyWith<$Res>? get mailingAddress;
}

/// @nodoc
final class _$ProfileCopyWithImpl<$Res> implements _$ProfileCopyWith<$Res> {
  const _$ProfileCopyWithImpl(this._self, this._then);

  final Profile _self;
  final $Res Function(Profile) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? name = null,
    Object? nickname = _profileCopyWithUnset,
    Object? address = null,
    Object? mailingAddress = _profileCopyWithUnset,
  }) {
    return _then(
      Profile(
        name: name == null ? _self.name : name as String,
        nickname: identical(nickname, _profileCopyWithUnset)
            ? _self.nickname
            : nickname as String?,
        address: address == null ? _self.address : address as Address,
        mailingAddress: identical(mailingAddress, _profileCopyWithUnset)
            ? _self.mailingAddress
            : mailingAddress as Address?,
      )
    );
  }

  @override
  @pragma('vm:prefer-inline')
  _$AddressCopyWith<$Res> get address {
    return _$AddressCopyWithImpl<$Res>(
      _self.address,
      (value) => call(address: value),
    );
  }

  @override
  @pragma('vm:prefer-inline')
  _$AddressCopyWith<$Res>? get mailingAddress {
    final mailingAddressValue = _self.mailingAddress;
    if (mailingAddressValue == null) {
      return null;
    }

    return _$AddressCopyWithImpl<$Res>(
      mailingAddressValue,
      (value) => call(mailingAddress: value),
    );
  }
}"#
            .to_owned(),
        ]
        .as_slice()
    );
}

#[test]
fn copywith_handles_named_model_collections_without_aliasing() {
    let plugin = register_plugin();
    let contribution = plugin
        .generate(
            &library(vec![
                class(
                    "Node",
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
                    "Graph",
                    vec![field("nodes", TypeIr::list_of(TypeIr::named("Node")))],
                    vec![constructor(
                        None,
                        vec![constructor_param(
                            "nodes",
                            TypeIr::list_of(TypeIr::named("Node")),
                            ParamKind::Positional,
                        )],
                    )],
                    &["dust_dart::CopyWith"],
                ),
            ]),
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    let members = members_for_class(&contribution, "Graph");
    assert_eq!(
        members,
        [r#"/// Creates a copy of this `Graph` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = graph.copyWith();
/// ```
@pragma('vm:prefer-inline')
_$GraphCopyWith<Graph> get copyWith => _$GraphCopyWithImpl<Graph>(this as Graph, (value) => value);"#
        .to_owned()]
        .as_slice()
    );
    let _ = span(0, 0);
}
