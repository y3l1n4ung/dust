use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, library};

#[test]
fn handles_special_builtin_types() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "Specials",
            vec![
                field("dt", TypeIr::named("DateTime")),
                field("u", TypeIr::named("Uri")),
                field("bi", TypeIr::named("BigInt")),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param("dt", TypeIr::named("DateTime"), ParamKind::Named),
                    constructor_param("u", TypeIr::named("Uri"), ParamKind::Named),
                    constructor_param("bi", TypeIr::named("BigInt"), ParamKind::Named),
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
        r#"Map<String, Object?> _$SpecialsToJson(Specials instance) {
  return <String, Object?>{
    'dt': instance.dt.toIso8601String(),
    'u': instance.u.toString(),
    'bi': instance.bi.toString(),
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"// factory Specials.fromJson(Map<String, Object?> json) => _$SpecialsFromJson(json);
Specials _$SpecialsFromJson(Map<String, Object?> json) {
  final dtValue = _jsonAsDateTime(json['dt'], 'dt');
  final uValue = _jsonAsUri(json['u'], 'u');
  final biValue = _jsonAsBigInt(json['bi'], 'bi');

  return Specials(dt: dtValue, u: uValue, bi: biValue);
}"#
    );
}
