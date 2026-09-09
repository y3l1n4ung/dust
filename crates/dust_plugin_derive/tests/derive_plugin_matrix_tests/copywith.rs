use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{
    class, constructor, constructor_param, field, library, members_for_class, span,
};

/// Option-valued and unknown fields in copyWith.
#[path = "copywith/options.rs"]
mod options;

/// copyWith over nested models, collisions and collections.
#[path = "copywith/nested.rs"]
mod nested;

#[test]
fn copywith_uses_named_arguments_without_braces_in_constructor_calls() {
    let plugin = register_plugin();
    let contribution = plugin
        .generate(
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
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
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
    let contribution = plugin
        .generate(
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
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

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
