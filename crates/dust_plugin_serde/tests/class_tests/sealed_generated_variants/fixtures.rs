//! Class fixtures the sealed-variant tests are built from.

use super::*;

pub(super) fn payment_event_base() -> ClassIr {
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

pub(super) fn shape_event_base() -> ClassIr {
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

pub(super) fn serde_base(name: &str) -> ClassIr {
    let mut base = class(
        name,
        Vec::new(),
        vec![constructor(None, Vec::new())],
        &["dust_dart::Serialize", "dust_dart::Deserialize"],
    );
    base.kind = ClassKindIr::SealedClass;
    base
}

pub(super) fn defaulted_constructor_param(
    name: &str,
    ty: TypeIr,
    default: &str,
) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(30, 35),
        kind: ParamKind::Named,
        has_default: true,
        default_value_source: Some(default.to_owned()),
    }
}

pub(super) fn success_variant() -> ClassIr {
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

pub(super) fn failed_variant() -> ClassIr {
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

pub(super) fn payment_event_variant_support() -> String {
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

pub(super) fn shape_variant_support() -> String {
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

pub(super) fn payment_event_to_json() -> &'static str {
    r#"Map<String, Object?> _$JsonPaymentEventSerialize(JsonPaymentEvent instance) {
  return switch (instance) {
    JsonPaymentSuccess value => <String, Object?>{
      ..._$JsonPaymentSuccessSerialize(value),
      'type': 'payment_success',
    },
    JsonPaymentFailed value => <String, Object?>{
      ..._$JsonPaymentFailedSerialize(value),
      'type': 'payment_failed',
    },
  };
}

Map<String, Object?> _$JsonPaymentEventToJson(JsonPaymentEvent instance) =>
    _$JsonPaymentEventSerialize(instance);"#
}

pub(super) fn success_to_json() -> &'static str {
    r#"Map<String, Object?> _$JsonPaymentSuccessSerialize(JsonPaymentSuccess instance) {
  return <String, Object?>{
    'id': instance.id,
    'cents': instance.cents,
    'currency': instance.currency,
  };
}

Map<String, Object?> _$JsonPaymentSuccessToJson(JsonPaymentSuccess instance) =>
    _$JsonPaymentSuccessSerialize(instance);"#
}
