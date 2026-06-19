use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use crate::support::{members_for_class, span};

#[test]
fn copywith_preserves_collection_references_without_deep_copy() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            package_root: ".".to_owned(),
            package_name: "dust_test".to_owned(),
            source_path: "lib/catalog.dart".to_owned(),
            output_path: "lib/catalog.g.dart".to_owned(),
            imports: Vec::new(),
            library: None,
            library_annotations: Vec::new(),
            import_directives: Vec::new(),
            export_directives: Vec::new(),
            part_directives: Vec::new(),
            part_of: None,
            span: span(0, 100),
            classes: vec![ClassIr {
                kind: ClassKindIr::Class,
                name: "Catalog".to_owned(),
                is_abstract: false,
                is_interface: false,
                superclass_name: None,
                span: span(10, 80),
                fields: vec![
                    FieldIr {
                        name: "groups".to_owned(),
                        ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                        span: span(18, 19),
                        has_default: false,
                        serde: None,
                        configs: Vec::new(),
                    },
                    FieldIr {
                        name: "items".to_owned(),
                        ty: TypeIr::list_of(TypeIr::string()),
                        span: span(20, 30),
                        has_default: false,
                        serde: None,
                        configs: Vec::new(),
                    },
                    FieldIr {
                        name: "tags".to_owned(),
                        ty: TypeIr::generic("Set", vec![TypeIr::string()]).nullable(),
                        span: span(31, 40),
                        has_default: false,
                        serde: None,
                        configs: Vec::new(),
                    },
                    FieldIr {
                        name: "metrics".to_owned(),
                        ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                        span: span(41, 50),
                        has_default: false,
                        serde: None,
                        configs: Vec::new(),
                    },
                ],
                constructors: vec![ConstructorIr {
                    name: None,
                    is_factory: false,
                    redirected_target_source: None,
                    redirected_target_name: None,
                    span: span(40, 60),
                    params: vec![
                        ConstructorParamIr {
                            name: "groups".to_owned(),
                            ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                            span: span(40, 41),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        },
                        ConstructorParamIr {
                            name: "items".to_owned(),
                            ty: TypeIr::list_of(TypeIr::string()),
                            span: span(42, 44),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        },
                        ConstructorParamIr {
                            name: "tags".to_owned(),
                            ty: TypeIr::generic("Set", vec![TypeIr::string()]).nullable(),
                            span: span(45, 47),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        },
                        ConstructorParamIr {
                            name: "metrics".to_owned(),
                            ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                            span: span(48, 49),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        },
                    ],
                }],
                methods: Vec::new(),
                traits: vec![TraitApplicationIr {
                    symbol: SymbolId::new("dust_dart::CopyWith"),
                    span: span(5, 9),
                }],
                configs: Vec::new(),
                serde: None,
            }],
            mixins: Vec::new(),
            extensions: Vec::new(),
            extension_types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
            enums: Vec::new(),
            query_calls: Vec::new(),
        },
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Catalog");

    assert_eq!(members.len(), 1);
    assert_eq!(
        members,
        [r#"/// Creates a copy of this `Catalog` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = catalog.copyWith();
/// final cleared = catalog.copyWith(tags: null);
/// ```
@pragma('vm:prefer-inline')
_$CatalogCopyWith<Catalog> get copyWith => _$CatalogCopyWithImpl<Catalog>(this as Catalog, (value) => value);"#
        .to_owned()]
        .as_slice()
    );
    assert_eq!(
        contribution.shared_helpers,
        [r#"final class _CatalogCopyWithUnset {
  const _CatalogCopyWithUnset();
}

const _catalogCopyWithUnset = _CatalogCopyWithUnset();"#
            .to_owned()]
        .as_slice()
    );
    assert_eq!(
        contribution.support_types,
        [r#"// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$CatalogCopyWith<$Res> {
  $Res call({
    List<List<String>>? groups,
    List<String>? items,
    Set<String>? tags,
    Map<String, List<int>>? metrics,
  });
}

/// @nodoc
final class _$CatalogCopyWithImpl<$Res> implements _$CatalogCopyWith<$Res> {
  const _$CatalogCopyWithImpl(this._self, this._then);

  final Catalog _self;
  final $Res Function(Catalog) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? groups = null,
    Object? items = null,
    Object? tags = _catalogCopyWithUnset,
    Object? metrics = null,
  }) {
    return _then(
      Catalog(
        groups == null ? _self.groups : groups as List<List<String>>,
        items == null ? _self.items : items as List<String>,
        identical(tags, _catalogCopyWithUnset)
            ? _self.tags
            : tags as Set<String>?,
        metrics == null ? _self.metrics : metrics as Map<String, List<int>>,
      )
    );
  }
}"#
        .to_owned()]
        .as_slice()
    );

    let generated = contribution.support_types.join("\n");
    assert!(!generated.contains("List.of"));
    assert!(!generated.contains("Set<String>.of"));
    assert!(!generated.contains("Map.fromEntries"));
}
