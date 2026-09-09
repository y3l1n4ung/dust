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

/// Enum wire names and typed default values.
#[path = "validation_tests/defaults.rs"]
mod defaults;

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
