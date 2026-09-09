//! copyWith over nested models, collisions and collections.

use super::*;

/// Nullable nested model helpers and named model collections.
#[path = "nested/helpers.rs"]
mod helpers;

#[test]
fn copywith_uses_stable_temp_bindings_for_nested_types() {
    let plugin = register_plugin();
    let contribution = plugin
        .generate(
            &library(vec![crate::support::class(
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
                vec![crate::support::constructor(
                    None,
                    vec![
                        constructor_param(
                            "left",
                            TypeIr::map_of(
                                TypeIr::string(),
                                TypeIr::list_of(TypeIr::named("Node")),
                            )
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
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
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
    let contribution = plugin
        .generate(
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
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

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
