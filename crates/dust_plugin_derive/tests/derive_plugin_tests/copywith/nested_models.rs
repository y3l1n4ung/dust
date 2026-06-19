use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use crate::support::{members_for_class, span};

#[test]
fn copywith_replaces_nested_models_shallowly_and_emits_chained_helper() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            package_root: ".".to_owned(),
            package_name: "dust_test".to_owned(),
            source_path: "lib/product.dart".to_owned(),
            output_path: "lib/product.g.dart".to_owned(),
            imports: Vec::new(),
            library: None,
            library_annotations: Vec::new(),
            import_directives: Vec::new(),
            export_directives: Vec::new(),
            part_directives: Vec::new(),
            part_of: None,
            span: span(0, 120),
            classes: vec![
                ClassIr {
                    kind: ClassKindIr::Class,
                    name: "Price".to_owned(),
                    is_abstract: false,
                    is_interface: false,
                    superclass_name: None,
                    span: span(1, 20),
                    fields: vec![FieldIr {
                        name: "currency".to_owned(),
                        ty: TypeIr::string(),
                        span: span(2, 3),
                        has_default: false,
                        serde: None,
                        configs: Vec::new(),
                    }],
                    constructors: vec![ConstructorIr {
                        name: None,
                        is_factory: false,
                        redirected_target_source: None,
                        redirected_target_name: None,
                        span: span(3, 4),
                        params: vec![ConstructorParamIr {
                            name: "currency".to_owned(),
                            ty: TypeIr::string(),
                            span: span(3, 4),
                            kind: ParamKind::Positional,
                            has_default: false,
                            default_value_source: None,
                        }],
                    }],
                    methods: Vec::new(),
                    traits: vec![TraitApplicationIr {
                        symbol: SymbolId::new("dust_dart::CopyWith"),
                        span: span(1, 2),
                    }],
                    configs: Vec::new(),
                    serde: None,
                },
                ClassIr {
                    kind: ClassKindIr::Class,
                    name: "Product".to_owned(),
                    is_abstract: false,
                    is_interface: false,
                    superclass_name: None,
                    span: span(20, 100),
                    fields: vec![
                        FieldIr {
                            name: "price".to_owned(),
                            ty: TypeIr::named("Price"),
                            span: span(21, 22),
                            has_default: false,
                            serde: None,
                            configs: Vec::new(),
                        },
                        FieldIr {
                            name: "prices".to_owned(),
                            ty: TypeIr::list_of(TypeIr::named("Price")),
                            span: span(22, 23),
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
                        span: span(24, 25),
                        params: vec![
                            ConstructorParamIr {
                                name: "price".to_owned(),
                                ty: TypeIr::named("Price"),
                                span: span(24, 25),
                                kind: ParamKind::Positional,
                                has_default: false,
                                default_value_source: None,
                            },
                            ConstructorParamIr {
                                name: "prices".to_owned(),
                                ty: TypeIr::list_of(TypeIr::named("Price")),
                                span: span(25, 26),
                                kind: ParamKind::Positional,
                                has_default: false,
                                default_value_source: None,
                            },
                        ],
                    }],
                    methods: Vec::new(),
                    traits: vec![TraitApplicationIr {
                        symbol: SymbolId::new("dust_dart::CopyWith"),
                        span: span(21, 22),
                    }],
                    configs: Vec::new(),
                    serde: None,
                },
            ],
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

    let members = members_for_class(&contribution, "Product");
    assert_eq!(members.len(), 1);
    assert_eq!(
        members,
        [r#"/// Creates a copy of this `Product` with selected fields replaced.
///
/// Usage:
/// ```dart
/// final updated = product.copyWith();
/// final nested = product.copyWith.price(currency: 'London');
/// ```
@pragma('vm:prefer-inline')
_$ProductCopyWith<Product> get copyWith => _$ProductCopyWithImpl<Product>(this as Product, (value) => value);"#
        .to_owned()]
        .as_slice()
    );
    assert_eq!(contribution.shared_helpers, Vec::<String>::new());
    assert_eq!(
        contribution.support_types,
        [
            r#"// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$PriceCopyWith<$Res> {
  $Res call({
    String? currency,
  });
}

/// @nodoc
final class _$PriceCopyWithImpl<$Res> implements _$PriceCopyWith<$Res> {
  const _$PriceCopyWithImpl(this._self, this._then);

  final Price _self;
  final $Res Function(Price) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? currency = null,
  }) {
    return _then(
      Price(
        currency == null ? _self.currency : currency as String,
      )
    );
  }
}"#
            .to_owned(),
            r#"/// @nodoc
abstract class _$ProductCopyWith<$Res> {
  $Res call({
    Price? price,
    List<Price>? prices,
  });

  _$PriceCopyWith<$Res> get price;
}

/// @nodoc
final class _$ProductCopyWithImpl<$Res> implements _$ProductCopyWith<$Res> {
  const _$ProductCopyWithImpl(this._self, this._then);

  final Product _self;
  final $Res Function(Product) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? price = null,
    Object? prices = null,
  }) {
    return _then(
      Product(
        price == null ? _self.price : price as Price,
        prices == null ? _self.prices : prices as List<Price>,
      )
    );
  }

  @override
  @pragma('vm:prefer-inline')
  _$PriceCopyWith<$Res> get price {
    return _$PriceCopyWithImpl<$Res>(
      _self.price,
      (value) => call(price: value),
    );
  }
}"#
            .to_owned(),
        ]
        .as_slice()
    );

    let generated = contribution.support_types.join("\n");
    assert!(!generated.contains("item_"));
    assert!(!generated.contains(".map((item"));
    assert!(!generated.contains("List<Price>.of"));
}
