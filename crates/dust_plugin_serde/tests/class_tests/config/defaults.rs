use dust_ir::{BuiltinType, ParamKind, SerdeFieldConfigIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use crate::support::{class, constructor, constructor_param, library, span};

#[test]
fn supports_default_values_during_deserialization() {
    let plugin = register_plugin();
    let user_class = class(
        "User",
        vec![dust_ir::FieldIr {
            name: "role".to_owned(),
            ty: TypeIr::string(),
            span: span(10, 20),
            has_default: false,
            serde: Some(SerdeFieldConfigIr {
                default_value_source: Some("'guest'".to_owned()),
                ..Default::default()
            }),
            configs: Vec::new(),
        }],
        vec![constructor(
            None,
            vec![constructor_param(
                "role",
                TypeIr::string(),
                ParamKind::Named,
            )],
        )],
        &["dust_dart::Deserialize"],
    );

    let library = library(vec![user_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        &contribution.top_level_functions[0],
        r#"// factory User.fromJson(Map<String, Object?> json) => _$UserFromJson(json);
User _$UserFromJson(Map<String, Object?> json) {
  final roleValue = json.containsKey('role')
      ? JsonHelper.as<String>(json['role'], 'role', 'String')
      : 'guest';

  return User(role: roleValue);
}"#
    );
}

#[test]
fn keeps_defaulted_double_decode_syntax_valid() {
    let plugin = register_plugin();
    let quote_class = class(
        "Quote",
        vec![dust_ir::FieldIr {
            name: "subtotal".to_owned(),
            ty: TypeIr::builtin(BuiltinType::Double),
            span: span(10, 20),
            has_default: false,
            serde: Some(SerdeFieldConfigIr {
                default_value_source: Some("0.0".to_owned()),
                ..Default::default()
            }),
            configs: Vec::new(),
        }],
        vec![constructor(
            None,
            vec![constructor_param(
                "subtotal",
                TypeIr::builtin(BuiltinType::Double),
                ParamKind::Named,
            )],
        )],
        &["dust_dart::Deserialize"],
    );

    let library = library(vec![quote_class], vec![]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    assert_eq!(
        &contribution.top_level_functions[0],
        r#"// factory Quote.fromJson(Map<String, Object?> json) => _$QuoteFromJson(json);
Quote _$QuoteFromJson(Map<String, Object?> json) {
  final subtotalValue = json.containsKey('subtotal')
      ? JsonHelper.as<num>(json['subtotal'], 'subtotal', 'num').toDouble()
      : 0.0;

  return Quote(subtotal: subtotalValue);
}"#
    );
}
