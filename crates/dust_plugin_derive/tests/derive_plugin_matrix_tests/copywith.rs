use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{
    class, constructor, constructor_param, field, library, members_for_class, span,
};

#[test]
fn copywith_uses_named_arguments_without_braces_in_constructor_calls() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Request",
            vec![
                field("path", TypeIr::string()),
                field(
                    "headers",
                    TypeIr::map_of(TypeIr::string(), TypeIr::string()),
                ),
            ],
            vec![constructor(
                Some("create"),
                vec![
                    constructor_param("path", TypeIr::string(), ParamKind::Named),
                    constructor_param(
                        "headers",
                        TypeIr::map_of(TypeIr::string(), TypeIr::string()),
                        ParamKind::Named,
                    ),
                ],
            )],
            &["dust_dart::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Request");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(contribution.shared_helpers, Vec::<String>::new());
    assert_eq!(
        members,
        [r#"/// Creates a copy of this `Request` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = request.copyWith(path: 'John');
/// ```
@pragma('vm:prefer-inline')
_$RequestCopyWith<Request> get copyWith => _$RequestCopyWithImpl<Request>(this as Request, (value) => value);"#
        .to_owned()]
        .as_slice()
    );
    assert_eq!(
        contribution.support_types,
        [r#"// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$RequestCopyWith<$Res> {
  $Res call({
    String? path,
    Map<String, String>? headers,
  });
}

/// @nodoc
final class _$RequestCopyWithImpl<$Res> implements _$RequestCopyWith<$Res> {
  const _$RequestCopyWithImpl(this._self, this._then);

  final Request _self;
  final $Res Function(Request) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? path = null,
    Object? headers = null,
  }) {
    return _then(
      Request.create(
        path: path == null ? _self.path : path as String,
        headers: headers == null ? _self.headers : headers as Map<String, String>,
      )
    );
  }
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_renders_nested_generic_and_dynamic_casts() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Payload",
            vec![
                field("items", TypeIr::list_of(TypeIr::string()).nullable()),
                field("extra", TypeIr::dynamic()),
                field("transform", TypeIr::function("void Function(String, int)")),
                field("summary", TypeIr::record("(String, int)")),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param(
                        "items",
                        TypeIr::list_of(TypeIr::string()).nullable(),
                        ParamKind::Positional,
                    ),
                    constructor_param("extra", TypeIr::dynamic(), ParamKind::Positional),
                    constructor_param(
                        "transform",
                        TypeIr::function("void Function(String, int)"),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "summary",
                        TypeIr::record("(String, int)"),
                        ParamKind::Positional,
                    ),
                ],
            )],
            &["dust_dart::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );

    let members = members_for_class(&contribution, "Payload");
    assert_eq!(
        members,
        [r#"/// Creates a copy of this `Payload` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = payload.copyWith();
/// final cleared = payload.copyWith(items: null);
/// ```
@pragma('vm:prefer-inline')
_$PayloadCopyWith<Payload> get copyWith => _$PayloadCopyWithImpl<Payload>(this as Payload, (value) => value);"#
        .to_owned()]
        .as_slice()
    );
    assert_eq!(
        contribution.shared_helpers,
        [r#"final class _PayloadCopyWithUnset {
  const _PayloadCopyWithUnset();
}

const _payloadCopyWithUnset = _PayloadCopyWithUnset();"#
            .to_owned()]
        .as_slice()
    );
    assert_eq!(
        contribution.support_types,
        [r#"// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$PayloadCopyWith<$Res> {
  $Res call({
    List<String>? items,
    dynamic extra,
    void Function(String, int)? transform,
    (String, int)? summary,
  });
}

/// @nodoc
final class _$PayloadCopyWithImpl<$Res> implements _$PayloadCopyWith<$Res> {
  const _$PayloadCopyWithImpl(this._self, this._then);

  final Payload _self;
  final $Res Function(Payload) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? items = _payloadCopyWithUnset,
    Object? extra = _payloadCopyWithUnset,
    Object? transform = null,
    Object? summary = null,
  }) {
    return _then(
      Payload(
        identical(items, _payloadCopyWithUnset)
            ? _self.items
            : items as List<String>?,
        identical(extra, _payloadCopyWithUnset)
            ? _self.extra
            : extra,
        transform == null ? _self.transform : transform as void Function(String, int),
        summary == null ? _self.summary : summary as (String, int),
      )
    );
  }
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_uses_option_update_for_option_and_unknown_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
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
        &SymbolPlan::default(),
    );

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

#[test]
fn copywith_uses_stable_temp_bindings_for_nested_types() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![super::support::class(
            "Complex",
            vec![
                field(
                    "left",
                    TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::named("Node")))
                        .nullable(),
                ),
                field(
                    "right",
                    TypeIr::generic("Set", vec![TypeIr::list_of(TypeIr::string())]),
                ),
            ],
            vec![super::support::constructor(
                None,
                vec![
                    constructor_param(
                        "left",
                        TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::named("Node")))
                            .nullable(),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "right",
                        TypeIr::generic("Set", vec![TypeIr::list_of(TypeIr::string())]),
                        ParamKind::Positional,
                    ),
                ],
            )],
            &["dust_dart::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Complex");
    assert_eq!(
        members,
        [r#"/// Creates a copy of this `Complex` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = complex.copyWith();
/// final cleared = complex.copyWith(left: null);
/// ```
@pragma('vm:prefer-inline')
_$ComplexCopyWith<Complex> get copyWith => _$ComplexCopyWithImpl<Complex>(this as Complex, (value) => value);"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_allocates_locals_around_field_name_collisions() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Collision",
            vec![
                field("_self", TypeIr::string()),
                field("_then", TypeIr::string()),
                field("value", TypeIr::string()),
                field("items", TypeIr::list_of(TypeIr::string()).nullable()),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param("_self", TypeIr::string(), ParamKind::Positional),
                    constructor_param("_then", TypeIr::string(), ParamKind::Positional),
                    constructor_param("value", TypeIr::string(), ParamKind::Positional),
                    constructor_param(
                        "items",
                        TypeIr::list_of(TypeIr::string()).nullable(),
                        ParamKind::Positional,
                    ),
                ],
            )],
            &["dust_dart::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );

    let members = members_for_class(&contribution, "Collision");
    assert_eq!(
        members,
        [r#"/// Creates a copy of this `Collision` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = collision.copyWith(_self: 'John');
/// final cleared = collision.copyWith(items: null);
/// ```
@pragma('vm:prefer-inline')
_$CollisionCopyWith<Collision> get copyWith => _$CollisionCopyWithImpl<Collision>(this as Collision, (value2) => value2);"#
        .to_owned()]
        .as_slice()
    );
    assert_eq!(
        contribution.support_types,
        [r#"// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$CollisionCopyWith<$Res> {
  $Res call({
    String? _self,
    String? _then,
    String? value,
    List<String>? items,
  });
}

/// @nodoc
final class _$CollisionCopyWithImpl<$Res> implements _$CollisionCopyWith<$Res> {
  const _$CollisionCopyWithImpl(this._self2, this._then2);

  final Collision _self2;
  final $Res Function(Collision) _then2;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? _self = null,
    Object? _then = null,
    Object? value = null,
    Object? items = _collisionCopyWithUnset,
  }) {
    return _then2(
      Collision(
        _self == null ? _self2._self : _self as String,
        _then == null ? _self2._then : _then as String,
        value == null ? _self2.value : value as String,
        identical(items, _collisionCopyWithUnset)
            ? _self2.items
            : items as List<String>?,
      )
    );
  }
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_emits_nullable_nested_model_helper() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
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
                        constructor_param("address", TypeIr::named("Address"), ParamKind::Named),
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
        &SymbolPlan::default(),
    );

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
    let contribution = plugin.emit(
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
        &SymbolPlan::default(),
    );

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
