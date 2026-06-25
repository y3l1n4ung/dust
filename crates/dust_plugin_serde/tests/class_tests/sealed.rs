use dust_ir::{ClassIr, ClassKindIr, ParamKind, SerdeClassConfigIr, SerdeVariantConfigIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, library, members_for_class};

#[test]
fn generates_internal_tagged_sealed_helpers() {
    let plugin = register_plugin();
    let library = library(
        vec![sealed_base(None), success_variant(), failed_variant()],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        members_for_class(&contribution, "JsonPaymentSuccess")[0],
        "Map<String, Object?> toJson() =>\n    _$JsonPaymentEventToJson(this as JsonPaymentEvent);"
    );
    assert_eq!(
        function_for(
            &contribution.top_level_functions,
            "_$JsonPaymentEventToJson"
        ),
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
    );
    assert_eq!(
        function_for(
            &contribution.top_level_functions,
            "_$JsonPaymentEventFromJson",
        ),
        r#"// factory JsonPaymentEvent.fromJson(Map<String, Object?> json) => _$JsonPaymentEventFromJson(json);
JsonPaymentEvent _$JsonPaymentEventFromJson(Map<String, Object?> json) {
  final tagValue = JsonHelper.as<String>(json['type'], 'type', 'String');
  final variantJson = Map<String, Object?>.from(json)..remove('type');

  return switch (tagValue) {
    'payment_success' => _$JsonPaymentSuccessFromJson(variantJson),
    'payment_failed' => _$JsonPaymentFailedFromJson(variantJson),
    _ => throw ArgumentError('Unknown SerDe variant tag: $tagValue'),
  };
}"#
    );
}

#[test]
fn generates_adjacent_tagged_sealed_helpers() {
    let plugin = register_plugin();
    let library = library(
        vec![
            sealed_base(Some("payload")),
            success_variant(),
            failed_variant(),
        ],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        function_for(
            &contribution.top_level_functions,
            "_$JsonPaymentEventToJson"
        ),
        r#"Map<String, Object?> _$JsonPaymentEventToJson(JsonPaymentEvent instance) {
  return switch (instance) {
    JsonPaymentSuccess value => <String, Object?>{
      'type': 'payment_success',
      'payload': _$JsonPaymentSuccessToJson(value),
    },
    JsonPaymentFailed value => <String, Object?>{
      'type': 'payment_failed',
      'payload': _$JsonPaymentFailedToJson(value),
    },
  };
}"#
    );
    assert_eq!(
        function_for(
            &contribution.top_level_functions,
            "_$JsonPaymentEventFromJson",
        ),
        r#"// factory JsonPaymentEvent.fromJson(Map<String, Object?> json) => _$JsonPaymentEventFromJson(json);
JsonPaymentEvent _$JsonPaymentEventFromJson(Map<String, Object?> json) {
  final tagValue = JsonHelper.as<String>(json['type'], 'type', 'String');
  final contentValue = JsonHelper.asMap(json['payload'], 'payload');

  return switch (tagValue) {
    'payment_success' => _$JsonPaymentSuccessFromJson(contentValue),
    'payment_failed' => _$JsonPaymentFailedFromJson(contentValue),
    _ => throw ArgumentError('Unknown SerDe variant tag: $tagValue'),
  };
}"#
    );
}

fn function_for<'a>(functions: &'a [String], needle: &str) -> &'a str {
    functions
        .iter()
        .find(|function| function.contains(needle))
        .map(String::as_str)
        .unwrap_or("")
}

fn sealed_base(content: Option<&str>) -> ClassIr {
    let mut base = class(
        "JsonPaymentEvent",
        Vec::new(),
        vec![constructor(None, Vec::new())],
        &["dust_dart::Serialize", "dust_dart::Deserialize"],
    );
    base.kind = ClassKindIr::SealedClass;
    base.serde = Some(SerdeClassConfigIr {
        tag: Some("type".to_owned()),
        content: content.map(str::to_owned),
        variants: vec![
            SerdeVariantConfigIr {
                constructor_name: "success".to_owned(),
                target_class_name: "JsonPaymentSuccess".to_owned(),
                tag: "payment_success".to_owned(),
            },
            SerdeVariantConfigIr {
                constructor_name: "failed".to_owned(),
                target_class_name: "JsonPaymentFailed".to_owned(),
                tag: "payment_failed".to_owned(),
            },
        ],
        ..SerdeClassConfigIr::default()
    });
    base
}

fn success_variant() -> ClassIr {
    let mut success = class(
        "JsonPaymentSuccess",
        vec![
            field("id", TypeIr::string()),
            field("cents", TypeIr::builtin(dust_ir::BuiltinType::Int)),
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("id", TypeIr::string(), ParamKind::Named),
                constructor_param(
                    "cents",
                    TypeIr::builtin(dust_ir::BuiltinType::Int),
                    ParamKind::Named,
                ),
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
