//! Enum wire names and typed default values.

use super::*;

#[test]
fn rejects_duplicate_enum_variant_wire_names() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library(
        vec![],
        vec![EnumIr {
            name: "Status".to_owned(),
            span: span(0, 20),
            variants: vec![
                EnumVariantIr {
                    name: "pending".to_owned(),
                    serde: None,
                    span: span(5, 10),
                },
                EnumVariantIr {
                    name: "queued".to_owned(),
                    serde: Some(SerdeEnumVariantConfigIr {
                        rename: Some("pending".to_owned()),
                        skip: false,
                    }),
                    span: span(12, 18),
                },
            ],
            traits: vec![TraitApplicationIr {
                symbol: SymbolId::new("dust_dart::Serialize"),
                span: span(1, 5),
            }],
            serde: None,
        }],
    ));
    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "enum `Status` maps variants `pending` and `queued` to duplicate SerDe value `pending`"
        ]
    );
}

#[test]
fn rejects_incompatible_typed_default_values() {
    let plugin = register_plugin();
    let target = class(
        "Defaults",
        vec![
            field_with_default("name", TypeIr::string(), "null", AnnotationValueIr::Null),
            field_with_default(
                "count",
                TypeIr::int(),
                "'guest'",
                AnnotationValueIr::String("guest".to_owned()),
            ),
            field_with_default(
                "enabled",
                TypeIr::bool(),
                "1",
                AnnotationValueIr::Number {
                    source: "1".to_owned(),
                    kind: AnnotationNumberKindIr::Int,
                },
            ),
            field_with_default(
                "tags",
                TypeIr::list_of(TypeIr::string()),
                "{'a': 'b'}",
                AnnotationValueIr::Map(Vec::new()),
            ),
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("name", TypeIr::string(), ParamKind::Named),
                constructor_param("count", TypeIr::int(), ParamKind::Named),
                constructor_param("enabled", TypeIr::bool(), ParamKind::Named),
                constructor_param("tags", TypeIr::list_of(TypeIr::string()), ParamKind::Named),
            ],
        )],
        &["dust_dart::Deserialize"],
    );

    let diagnostics = plugin.validate(&library(vec![target], vec![]));
    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "field `name` on class `Defaults` uses `SerDe(defaultValue: null)` on non-nullable field",
            "field `count` on class `Defaults` uses `SerDe(defaultValue: 'guest')` that is not compatible with `int`",
            "field `enabled` on class `Defaults` uses `SerDe(defaultValue: 1)` that is not compatible with `bool`",
            "field `tags` on class `Defaults` uses `SerDe(defaultValue: {'a': 'b'})` that is not compatible with `List`",
        ]
    );
}

#[test]
fn accepts_compatible_typed_default_values() {
    let plugin = register_plugin();
    let target = class(
        "Defaults",
        vec![
            field_with_default(
                "name",
                TypeIr::string(),
                "'guest'",
                AnnotationValueIr::String("guest".to_owned()),
            ),
            field_with_default(
                "optional",
                TypeIr::string().nullable(),
                "null",
                AnnotationValueIr::Null,
            ),
            field_with_default(
                "enabled",
                TypeIr::bool(),
                "true",
                AnnotationValueIr::Bool(true),
            ),
            field_with_default(
                "count",
                TypeIr::int(),
                "1",
                AnnotationValueIr::Number {
                    source: "1".to_owned(),
                    kind: AnnotationNumberKindIr::Int,
                },
            ),
            field_with_default(
                "subtotal",
                TypeIr::double(),
                "1",
                AnnotationValueIr::Number {
                    source: "1".to_owned(),
                    kind: AnnotationNumberKindIr::Int,
                },
            ),
            field_with_default(
                "ratio",
                TypeIr::num(),
                "1.5",
                AnnotationValueIr::Number {
                    source: "1.5".to_owned(),
                    kind: AnnotationNumberKindIr::Double,
                },
            ),
            field_with_default(
                "tags",
                TypeIr::list_of(TypeIr::string()),
                "['a']",
                AnnotationValueIr::List(Vec::new()),
            ),
            field_with_default(
                "flags",
                TypeIr::generic("Set", vec![TypeIr::string()]),
                "{'a'}",
                AnnotationValueIr::Set(Vec::new()),
            ),
            field_with_default(
                "lookup",
                TypeIr::map_of(TypeIr::string(), TypeIr::string()),
                "{'a': 'b'}",
                AnnotationValueIr::Map(Vec::new()),
            ),
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("name", TypeIr::string(), ParamKind::Named),
                constructor_param("optional", TypeIr::string().nullable(), ParamKind::Named),
                constructor_param("enabled", TypeIr::bool(), ParamKind::Named),
                constructor_param("count", TypeIr::int(), ParamKind::Named),
                constructor_param("subtotal", TypeIr::double(), ParamKind::Named),
                constructor_param("ratio", TypeIr::num(), ParamKind::Named),
                constructor_param("tags", TypeIr::list_of(TypeIr::string()), ParamKind::Named),
                constructor_param(
                    "flags",
                    TypeIr::generic("Set", vec![TypeIr::string()]),
                    ParamKind::Named,
                ),
                constructor_param(
                    "lookup",
                    TypeIr::map_of(TypeIr::string(), TypeIr::string()),
                    ParamKind::Named,
                ),
            ],
        )],
        &["dust_dart::Deserialize"],
    );

    assert_eq!(plugin.validate(&library(vec![target], vec![])), Vec::new());
}
