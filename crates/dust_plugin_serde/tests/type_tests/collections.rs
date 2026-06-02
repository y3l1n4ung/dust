use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, library};

#[test]
fn handles_collection_types() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "Collections",
            vec![
                field("l", TypeIr::generic("List", vec![TypeIr::string()])),
                field("s", TypeIr::generic("Set", vec![TypeIr::string()])),
                field(
                    "m",
                    TypeIr::generic("Map", vec![TypeIr::string(), TypeIr::string()]),
                ),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param(
                        "l",
                        TypeIr::generic("List", vec![TypeIr::string()]),
                        ParamKind::Named,
                    ),
                    constructor_param(
                        "s",
                        TypeIr::generic("Set", vec![TypeIr::string()]),
                        ParamKind::Named,
                    ),
                    constructor_param(
                        "m",
                        TypeIr::generic("Map", vec![TypeIr::string(), TypeIr::string()]),
                        ParamKind::Named,
                    ),
                ],
            )],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert_eq!(
        to_json,
        r#"Map<String, Object?> _$CollectionsToJson(Collections instance) {
  return <String, Object?>{
    'l': instance.l
        .map((item) => item)
        .toList(),
    's': instance.s
        .map((item) => item)
        .toList(),
    'm': instance.m
        .map((key, value) => MapEntry(key, value)),
  };
}"#
    );
    assert_eq!(
        from_json,
        r#"// factory Collections.fromJson(Map<String, Object?> json) => _$CollectionsFromJson(json);
Collections _$CollectionsFromJson(Map<String, Object?> json) {
  final lValue = _jsonAsList(json['l'], 'l')
      .map((item) => _jsonAs<String>(item, 'l', 'String'))
      .toList();
  final sValue = _jsonAsList(json['s'], 's')
      .map((item) => _jsonAs<String>(item, 's', 'String'))
      .toSet();
  final mValue = _jsonAsMap(json['m'], 'm')
      .map((mapKey, value) => MapEntry(mapKey, _jsonAs<String>(value, 'm', 'String')));

  return Collections(l: lValue, s: sValue, m: mValue);
}"#
    );
}
