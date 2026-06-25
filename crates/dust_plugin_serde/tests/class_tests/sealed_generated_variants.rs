use dust_ir::{
    ClassIr, ClassKindIr, ConstructorParamIr, ParamKind, SerdeClassConfigIr, SerdeVariantConfigIr,
    TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, function_for, library, span};

#[test]
fn generates_missing_concrete_variant_classes_from_factories() {
    let plugin = register_plugin();
    let library = library(vec![payment_event_base()], vec![]);

    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        contribution.support_types,
        vec![payment_event_variant_support()]
    );
    assert_eq!(
        function_for(
            &contribution.top_level_functions,
            "_$JsonPaymentEventToJson",
        ),
        payment_event_to_json()
    );
    assert_eq!(
        function_for(
            &contribution.top_level_functions,
            "_$JsonPaymentSuccessToJson",
        ),
        success_to_json()
    );
}

#[test]
fn does_not_generate_source_defined_variant_classes() {
    let plugin = register_plugin();
    let library = library(
        vec![payment_event_base(), success_variant(), failed_variant()],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(contribution.support_types, Vec::<String>::new());
}

#[test]
fn renders_empty_positional_nullable_and_defaulted_variant_params() {
    let plugin = register_plugin();
    let library = library(vec![shape_event_base()], vec![]);

    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(contribution.support_types, vec![shape_variant_support()]);
}

fn payment_event_base() -> ClassIr {
    let mut base = serde_base("JsonPaymentEvent");
    base.serde = Some(SerdeClassConfigIr {
        tag: Some("type".to_owned()),
        variants: vec![
            SerdeVariantConfigIr {
                constructor_name: "success".to_owned(),
                target_class_name: "JsonPaymentSuccess".to_owned(),
                tag: "payment_success".to_owned(),
                params: vec![
                    constructor_param("id", TypeIr::string(), ParamKind::Named),
                    constructor_param("cents", TypeIr::int(), ParamKind::Named),
                    constructor_param("currency", TypeIr::string(), ParamKind::Named),
                ],
            },
            SerdeVariantConfigIr {
                constructor_name: "failed".to_owned(),
                target_class_name: "JsonPaymentFailed".to_owned(),
                tag: "payment_failed".to_owned(),
                params: vec![
                    constructor_param("id", TypeIr::string(), ParamKind::Named),
                    constructor_param("reason", TypeIr::string(), ParamKind::Named),
                    constructor_param("retryable", TypeIr::bool(), ParamKind::Named),
                ],
            },
        ],
        ..SerdeClassConfigIr::default()
    });
    base
}

fn shape_event_base() -> ClassIr {
    let mut base = serde_base("VariantShapeEvent");
    base.serde = Some(SerdeClassConfigIr {
        tag: Some("type".to_owned()),
        variants: vec![
            SerdeVariantConfigIr {
                constructor_name: "empty".to_owned(),
                target_class_name: "EmptyVariant".to_owned(),
                tag: "empty".to_owned(),
                params: Vec::new(),
            },
            SerdeVariantConfigIr {
                constructor_name: "mixed".to_owned(),
                target_class_name: "MixedVariant".to_owned(),
                tag: "mixed".to_owned(),
                params: vec![
                    constructor_param("id", TypeIr::string(), ParamKind::Positional),
                    constructor_param("note", TypeIr::string().nullable(), ParamKind::Named),
                    defaulted_constructor_param("retryCount", TypeIr::int(), "3"),
                ],
            },
        ],
        ..SerdeClassConfigIr::default()
    });
    base
}

fn serde_base(name: &str) -> ClassIr {
    let mut base = class(
        name,
        Vec::new(),
        vec![constructor(None, Vec::new())],
        &["dust_dart::Serialize", "dust_dart::Deserialize"],
    );
    base.kind = ClassKindIr::SealedClass;
    base
}

fn defaulted_constructor_param(name: &str, ty: TypeIr, default: &str) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(30, 35),
        kind: ParamKind::Named,
        has_default: true,
        default_value_source: Some(default.to_owned()),
    }
}

fn success_variant() -> ClassIr {
    let mut success = class(
        "JsonPaymentSuccess",
        vec![field("id", TypeIr::string()), field("cents", TypeIr::int())],
        vec![constructor(
            None,
            vec![
                constructor_param("id", TypeIr::string(), ParamKind::Named),
                constructor_param("cents", TypeIr::int(), ParamKind::Named),
            ],
        )],
        &["dust_dart::Serialize", "dust_dart::Deserialize"],
    );
    success.superclass_name = Some("JsonPaymentEvent".to_owned());
    success
}

fn failed_variant() -> ClassIr {
    let mut failed = class(
        "JsonPaymentFailed",
        vec![
            field("id", TypeIr::string()),
            field("reason", TypeIr::string()),
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("id", TypeIr::string(), ParamKind::Named),
                constructor_param("reason", TypeIr::string(), ParamKind::Named),
            ],
        )],
        &["dust_dart::Serialize", "dust_dart::Deserialize"],
    );
    failed.superclass_name = Some("JsonPaymentEvent".to_owned());
    failed
}

fn payment_event_variant_support() -> String {
    r#"final class JsonPaymentSuccess extends JsonPaymentEvent {
  const JsonPaymentSuccess({
    required this.id,
    required this.cents,
    required this.currency,
  }) : super();

  factory JsonPaymentSuccess.fromJson(Map<String, Object?> json) =>
      _$JsonPaymentSuccessFromJson(json);

  final String id;
  final int cents;
  final String currency;
}

final class JsonPaymentFailed extends JsonPaymentEvent {
  const JsonPaymentFailed({
    required this.id,
    required this.reason,
    required this.retryable,
  }) : super();

  factory JsonPaymentFailed.fromJson(Map<String, Object?> json) =>
      _$JsonPaymentFailedFromJson(json);

  final String id;
  final String reason;
  final bool retryable;
}"#
    .to_owned()
}

fn shape_variant_support() -> String {
    r#"final class EmptyVariant extends VariantShapeEvent {
  const EmptyVariant() : super();

  factory EmptyVariant.fromJson(Map<String, Object?> json) =>
      _$EmptyVariantFromJson(json);
}

final class MixedVariant extends VariantShapeEvent {
  const MixedVariant(
    this.id,
    {
      this.note,
      this.retryCount = 3,
    },
  ) : super();

  factory MixedVariant.fromJson(Map<String, Object?> json) =>
      _$MixedVariantFromJson(json);

  final String id;
  final String? note;
  final int retryCount;
}"#
    .to_owned()
}

fn payment_event_to_json() -> &'static str {
    r#"Map<String, Object?> _$JsonPaymentEventToJson(JsonPaymentEvent instance) {
  return switch (instance) {
    JsonPaymentSuccess value => <String, Object?>{
      ..._$JsonPaymentSuccessToJson(value),
      'type': 'payment_success',
    },
    JsonPaymentFailed value => <String, Object?>{
      ..._$JsonPaymentFailedToJson(value),
      'type': 'payment_failed',
    },
  };
}"#
}

fn success_to_json() -> &'static str {
    r#"Map<String, Object?> _$JsonPaymentSuccessToJson(JsonPaymentSuccess instance) {
  return <String, Object?>{
    'id': instance.id,
    'cents': instance.cents,
    'currency': instance.currency,
  };
}"#
}
