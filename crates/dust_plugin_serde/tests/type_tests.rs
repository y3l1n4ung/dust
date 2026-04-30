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
    }
}

fn constructor(name: Option<&str>, params: Vec<ConstructorParamIr>) -> ConstructorIr {
    ConstructorIr {
        name: name.map(str::to_owned),
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
        superclass_name: None,
        span: span(0, 100),
        fields,
        constructors,
        traits: traits
            .iter()
            .map(|symbol| trait_application(symbol))
            .collect(),
        serde: None,
    }
}

fn library(classes: Vec<ClassIr>, enums: Vec<EnumIr>) -> LibraryIr {
    LibraryIr {
        source_path: "lib/models.dart".to_owned(),
        output_path: "lib/models.g.dart".to_owned(),
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
                    constructor_param("i", TypeIr::builtin(dust_ir::BuiltinType::Int), ParamKind::Named),
                    constructor_param("b", TypeIr::builtin(dust_ir::BuiltinType::Bool), ParamKind::Named),
                    constructor_param(
                        "d",
                        TypeIr::builtin(dust_ir::BuiltinType::Double),
                        ParamKind::Named,
                    ),
                    constructor_param("n", TypeIr::builtin(dust_ir::BuiltinType::Num), ParamKind::Named),
                    constructor_param(
                        "o",
                        TypeIr::builtin(dust_ir::BuiltinType::Object),
                        ParamKind::Named,
                    ),
                    constructor_param("dyn", TypeIr::dynamic(), ParamKind::Named),
                ],
            )],
            &["derive_serde_annotation::Serialize", "derive_serde_annotation::Deserialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert!(to_json.contains("'s': instance.s,"));
    assert!(to_json.contains("'i': instance.i,"));
    assert!(to_json.contains("'b': instance.b,"));
    assert!(to_json.contains("'d': instance.d,"));
    assert!(to_json.contains("'n': instance.n,"));
    assert!(to_json.contains("'o': instance.o,"));
    assert!(to_json.contains("'dyn': instance.dyn,"));

    assert!(from_json.contains("final sValue = _dustJsonAs<String>(json['s'], 's', 'String');"));
    assert!(from_json.contains("final iValue = _dustJsonAs<int>(json['i'], 'i', 'int');"));
    assert!(from_json.contains("final bValue = _dustJsonAs<bool>(json['b'], 'b', 'bool');"));
    assert!(from_json.contains("final dValue = _dustJsonAs<num>(json['d'], 'd', 'num').toDouble();"));
    assert!(from_json.contains("final nValue = _dustJsonAs<num>(json['n'], 'n', 'num');"));
    assert!(from_json.contains("final oValue = _dustJsonAs<Object>(json['o'], 'o', 'Object');"));
    assert!(from_json.contains("final dynValue = json['dyn'];"));
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
            &["derive_serde_annotation::Serialize", "derive_serde_annotation::Deserialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert!(to_json.contains("'dt': instance.dt.toIso8601String(),"));
    assert!(to_json.contains("'u': instance.u.toString(),"));
    assert!(to_json.contains("'bi': instance.bi.toString(),"));

    assert!(from_json.contains("final dtValue = _dustJsonAsDateTime(json['dt'], 'dt');"));
    assert!(from_json.contains("final uValue = _dustJsonAsUri(json['u'], 'u');"));
    assert!(from_json.contains("final biValue = _dustJsonAsBigInt(json['bi'], 'bi');"));
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
                    constructor_param("l", TypeIr::generic("List", vec![TypeIr::string()]), ParamKind::Named),
                    constructor_param("s", TypeIr::generic("Set", vec![TypeIr::string()]), ParamKind::Named),
                    constructor_param(
                        "m",
                        TypeIr::generic("Map", vec![TypeIr::string(), TypeIr::string()]),
                        ParamKind::Named,
                    ),
                ],
            )],
            &["derive_serde_annotation::Serialize", "derive_serde_annotation::Deserialize"],
        )],
        vec![],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert!(to_json.contains("'l': instance.l.map((item) => item).toList(),"));
    assert!(to_json.contains("'s': instance.s.map((item) => item).toList(),"));
    assert!(to_json.contains("'m': instance.m.map((key, value) => MapEntry(key, value)),"));

    assert!(from_json.contains("final lValue = _dustJsonAsList(json['l'], 'l').map((item) => _dustJsonAs<String>(item, 'l', 'String')).toList();"));
    assert!(from_json.contains("final sValue = _dustJsonAsList(json['s'], 's').map((item) => _dustJsonAs<String>(item, 's', 'String')).toSet();"));
    assert!(from_json.contains("final mValue = _dustJsonAsMap(json['m'], 'm').map((mapKey, value) => MapEntry(mapKey, _dustJsonAs<String>(value, 'm', 'String')));"));
}
