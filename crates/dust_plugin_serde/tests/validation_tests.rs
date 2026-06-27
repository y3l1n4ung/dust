//! Integration tests for serde plugin validation diagnostics.

use dust_ir::{
    AnnotationNumberKindIr, AnnotationValueIr, EnumIr, EnumVariantIr, ParamKind,
    SerdeEnumVariantConfigIr, SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::DustPlugin;
use dust_plugin_serde::register_plugin;

use crate::support::{
    class, constructor, constructor_param, factory_constructor, field, field_with_default, library,
    method, span,
};

/// Fixture helpers for validation tests.
#[path = "validation_tests/support.rs"]
mod support;

#[test]
fn validates_abstract_deserialize_and_unsupported_field_types() {
    let plugin = register_plugin();
    let mut target = class(
        "Payload",
        vec![
            field("id", TypeIr::string()),
            field("transform", TypeIr::function("void Function(String)")),
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("id", TypeIr::string(), ParamKind::Positional),
                constructor_param(
                    "transform",
                    TypeIr::function("void Function(String)"),
                    ParamKind::Positional,
                ),
            ],
        )],
        &["dust_dart::Deserialize"],
    );
    target.is_abstract = true;

    let diagnostics = plugin.validate(&library(vec![target], vec![]));

    assert!(diagnostics.iter().any(|item| {
        item.message
            .contains("`Deserialize` cannot target abstract class `Payload`")
    }));
    assert!(diagnostics.iter().any(|item| {
        item.message
            .contains("`Deserialize` does not support function types on `Payload.transform`")
    }));
}

#[test]
fn validates_missing_deserialize_constructor() {
    let plugin = register_plugin();
    let target = class(
        "Payload",
        vec![field("id", TypeIr::string())],
        Vec::new(),
        &["dust_dart::Deserialize"],
    );

    let diagnostics = plugin.validate(&library(vec![target], vec![]));

    assert!(diagnostics.iter().any(|item| {
        item.message
            .contains("`Deserialize` requires a constructor that can initialize every field on class `Payload`")
    }));
}

#[test]
fn rejects_unverified_local_model_conversions() {
    let plugin = register_plugin();
    let target = class(
        "Payload",
        vec![field("profile", TypeIr::named("ExternalProfile"))],
        vec![constructor(
            None,
            vec![constructor_param(
                "profile",
                TypeIr::named("ExternalProfile"),
                ParamKind::Named,
            )],
        )],
        &["dust_dart::Serialize", "dust_dart::Deserialize"],
    );

    let external = class("ExternalProfile", Vec::new(), Vec::new(), &[]);
    let diagnostics = plugin.validate(&library(vec![target, external], vec![]));
    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "`Serialize` requires `ExternalProfile.toJson()` or deriving `Serialize`/using `SerDe(codec: ...)` for `Payload.profile`",
            "`Deserialize` requires `ExternalProfile.fromJson(Map<String, Object?>)` or deriving `Deserialize`/using `SerDe(codec: ...)` for `Payload.profile`",
        ]
    );
}

#[test]
fn accepts_local_json_capable_model_conversions() {
    let plugin = register_plugin();
    let target = class(
        "Payload",
        vec![field("profile", TypeIr::named("ExternalProfile"))],
        vec![constructor(
            None,
            vec![constructor_param(
                "profile",
                TypeIr::named("ExternalProfile"),
                ParamKind::Named,
            )],
        )],
        &["dust_dart::Serialize", "dust_dart::Deserialize"],
    );
    let mut external = class(
        "ExternalProfile",
        Vec::new(),
        vec![factory_constructor(
            Some("fromJson"),
            vec![constructor_param(
                "json",
                TypeIr::map_of(TypeIr::string(), TypeIr::object().nullable()),
                ParamKind::Positional,
            )],
        )],
        &[],
    );
    external.methods = vec![method(
        "toJson",
        TypeIr::map_of(TypeIr::string(), TypeIr::object().nullable()),
        Vec::new(),
    )];

    assert_eq!(
        plugin.validate(&library(vec![target, external], vec![])),
        Vec::new()
    );
}

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
