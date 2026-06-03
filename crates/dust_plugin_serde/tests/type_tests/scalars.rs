use dust_ir::{BuiltinType, ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, library};

#[test]
fn handles_scalar_builtin_types() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "Scalars",
            vec![
                field("s", TypeIr::string()),
                field("i", TypeIr::builtin(BuiltinType::Int)),
                field("b", TypeIr::builtin(BuiltinType::Bool)),
                field("d", TypeIr::builtin(BuiltinType::Double)),
                field("n", TypeIr::builtin(BuiltinType::Num)),
                field("o", TypeIr::builtin(BuiltinType::Object)),
                field("dyn", TypeIr::dynamic()),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param("s", TypeIr::string(), ParamKind::Named),
                    constructor_param("i", TypeIr::builtin(BuiltinType::Int), ParamKind::Named),
                    constructor_param("b", TypeIr::builtin(BuiltinType::Bool), ParamKind::Named),
                    constructor_param("d", TypeIr::builtin(BuiltinType::Double), ParamKind::Named),
                    constructor_param("n", TypeIr::builtin(BuiltinType::Num), ParamKind::Named),
                    constructor_param("o", TypeIr::builtin(BuiltinType::Object), ParamKind::Named),
                    constructor_param("dyn", TypeIr::dynamic(), ParamKind::Named),
                ],
            )],
            &["dust_dart::Serialize", "dust_dart::Deserialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Map<String, Object?> _$ScalarsToJson(Scalars instance) {
  return <String, Object?>{
    's': instance.s,
    'i': instance.i,
    'b': instance.b,
    'd': instance.d,
    'n': instance.n,
    'o': instance.o,
    'dyn': instance.dyn,
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"// factory Scalars.fromJson(Map<String, Object?> json) => _$ScalarsFromJson(json);
Scalars _$ScalarsFromJson(Map<String, Object?> json) {
  final sValue = _jsonAs<String>(json['s'], 's', 'String');
  final iValue = _jsonAs<int>(json['i'], 'i', 'int');
  final bValue = _jsonAs<bool>(json['b'], 'b', 'bool');
  final dValue = _jsonAs<num>(json['d'], 'd', 'num').toDouble();
  final nValue = _jsonAs<num>(json['n'], 'n', 'num');
  final oValue = _jsonAs<Object>(json['o'], 'o', 'Object');
  final dynValue = json['dyn'];

  return Scalars(
    s: sValue,
    i: iValue,
    b: bValue,
    d: dValue,
    n: nValue,
    o: oValue,
    dyn: dynValue,
  );
}"#
    );
}
