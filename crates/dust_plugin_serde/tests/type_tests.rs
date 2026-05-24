use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, EnumIr, FieldIr, LibraryIr, ParamKind,
    SpanIr, SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(11), TextRange::new(start, end))
}

fn trait_application(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
        span: span(1, 5),
    }
}

fn field(name: &str, ty: TypeIr) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(10, 20),
        has_default: false,
        serde: None,
    }
}

fn constructor_param(name: &str, ty: TypeIr, kind: ParamKind) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(30, 35),
        kind,
        has_default: false,
        default_value_source: None,
    }
}

fn constructor(name: Option<&str>, params: Vec<ConstructorParamIr>) -> ConstructorIr {
    ConstructorIr {
        name: name.map(str::to_owned),
        is_factory: false,
        redirected_target_source: None,
        redirected_target_name: None,
        span: span(25, 60),
        params,
    }
}

fn class(
    name: &str,
    fields: Vec<FieldIr>,
    constructors: Vec<ConstructorIr>,
    traits: &[&str],
) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(0, 100),
        fields,
        constructors,
        methods: Vec::new(),
        traits: traits
            .iter()
            .map(|symbol| trait_application(symbol))
            .collect(),
        configs: Vec::new(),
        serde: None,
    }
}

fn library(classes: Vec<ClassIr>, enums: Vec<EnumIr>) -> LibraryIr {
    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "dust_test".to_owned(),
        source_path: "lib/models.dart".to_owned(),
        output_path: "lib/models.g.dart".to_owned(),
        imports: Vec::new(),
        span: span(0, 200),
        classes,
        enums,
    }
}

#[test]
fn handles_scalar_builtin_types() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "Scalars",
            vec![
                field("s", TypeIr::string()),
                field("i", TypeIr::builtin(dust_ir::BuiltinType::Int)),
                field("b", TypeIr::builtin(dust_ir::BuiltinType::Bool)),
                field("d", TypeIr::builtin(dust_ir::BuiltinType::Double)),
                field("n", TypeIr::builtin(dust_ir::BuiltinType::Num)),
                field("o", TypeIr::builtin(dust_ir::BuiltinType::Object)),
                field("dyn", TypeIr::dynamic()),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param("s", TypeIr::string(), ParamKind::Named),
                    constructor_param(
                        "i",
                        TypeIr::builtin(dust_ir::BuiltinType::Int),
                        ParamKind::Named,
                    ),
                    constructor_param(
                        "b",
                        TypeIr::builtin(dust_ir::BuiltinType::Bool),
                        ParamKind::Named,
                    ),
                    constructor_param(
                        "d",
                        TypeIr::builtin(dust_ir::BuiltinType::Double),
                        ParamKind::Named,
                    ),
                    constructor_param(
                        "n",
                        TypeIr::builtin(dust_ir::BuiltinType::Num),
                        ParamKind::Named,
                    ),
                    constructor_param(
                        "o",
                        TypeIr::builtin(dust_ir::BuiltinType::Object),
                        ParamKind::Named,
                    ),
                    constructor_param("dyn", TypeIr::dynamic(), ParamKind::Named),
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
