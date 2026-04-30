use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{
    class, constructor, constructor_param, field, library, members_for_class, span,
};

#[test]
fn copywith_uses_named_arguments_without_braces_in_constructor_calls() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Request",
            vec![
                field("path", TypeIr::string()),
                field(
                    "headers",
                    TypeIr::map_of(TypeIr::string(), TypeIr::string()),
                ),
            ],
            vec![constructor(
                Some("create"),
                vec![
                    constructor_param("path", TypeIr::string(), ParamKind::Named),
                    constructor_param(
                        "headers",
                        TypeIr::map_of(TypeIr::string(), TypeIr::string()),
                        ParamKind::Named,
                    ),
                ],
            )],
            &["derive_annotation::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Request");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert!(members[0].contains("Request copyWith({"));
    assert!(members[0].contains("String? path,"));
    assert!(members[0].contains("Map<String, String>? headers,"));
    assert!(!members[0].contains("final nextPathSource = path ?? _dustSelf.path;"));
    assert!(!members[0].contains("final nextHeadersSource = headers ?? _dustSelf.headers;"));
    assert!(
        members[0]
            .contains("final nextHeaders = Map<String, String>.of(headers ?? _dustSelf.headers);")
    );
    assert!(members[0].contains("return Request.create("));
    assert!(members[0].contains("path: path ?? _dustSelf.path,"));
    assert!(members[0].contains("headers: nextHeaders,"));
    assert!(!members[0].contains("Request.create({"));
}

#[test]
fn copywith_renders_nested_generic_and_dynamic_casts() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Payload",
            vec![
                field("items", TypeIr::list_of(TypeIr::string()).nullable()),
                field("extra", TypeIr::dynamic()),
                field("transform", TypeIr::function("void Function(String, int)")),
                field("summary", TypeIr::record("(String, int)")),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param(
                        "items",
                        TypeIr::list_of(TypeIr::string()).nullable(),
                        ParamKind::Positional,
                    ),
                    constructor_param("extra", TypeIr::dynamic(), ParamKind::Positional),
                    constructor_param(
                        "transform",
                        TypeIr::function("void Function(String, int)"),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "summary",
                        TypeIr::record("(String, int)"),
                        ParamKind::Positional,
                    ),
                ],
            )],
            &["derive_annotation::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );

    let fragment = &members_for_class(&contribution, "Payload")[0];
    assert!(fragment.contains("Object? items = _undefined,"));
    assert!(fragment.contains("items as List<String>?"));
    assert!(fragment.contains("dynamic extra = _undefined,"));
    assert!(fragment.contains("extra as dynamic"));
    assert!(fragment.contains("void Function(String, int)? transform,"));
    assert!(!fragment.contains("final nextTransformSource = transform ?? _dustSelf.transform;"));
    assert!(fragment.contains("transform ?? _dustSelf.transform,"));
    assert!(fragment.contains("(String, int)? summary,"));
    assert!(!fragment.contains("final nextSummarySource = summary ?? _dustSelf.summary;"));
    assert!(fragment.contains("summary ?? _dustSelf.summary,"));
    assert!(fragment.contains("_dustSelf.items"));
}

#[test]
fn copywith_uses_stable_temp_bindings_for_nested_types() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![super::support::class(
            "Complex",
            vec![
                field(
                    "left",
                    TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::named("Node")))
                        .nullable(),
                ),
                field(
                    "right",
                    TypeIr::generic("Set", vec![TypeIr::list_of(TypeIr::string())]),
                ),
            ],
            vec![super::support::constructor(
                None,
                vec![
                    constructor_param(
                        "left",
                        TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::named("Node")))
                            .nullable(),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "right",
                        TypeIr::generic("Set", vec![TypeIr::list_of(TypeIr::string())]),
                        ParamKind::Positional,
                    ),
                ],
            )],
            &["derive_annotation::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );
    let fragment = &members_for_class(&contribution, "Complex")[0];

    assert!(fragment.contains("final nextLeftSource ="));
    assert!(fragment.contains("entry_0"));
    assert!(fragment.contains("item_1"));
    assert!(fragment.contains("item_"));
}

#[test]
fn copywith_handles_named_model_collections_without_aliasing() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![
            class(
                "Node",
                vec![field("id", TypeIr::string())],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "id",
                        TypeIr::string(),
                        ParamKind::Positional,
                    )],
                )],
                &["derive_annotation::CopyWith"],
            ),
            class(
                "Graph",
                vec![field("nodes", TypeIr::list_of(TypeIr::named("Node")))],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "nodes",
                        TypeIr::list_of(TypeIr::named("Node")),
                        ParamKind::Positional,
                    )],
                )],
                &["derive_annotation::CopyWith"],
            ),
        ]),
        &SymbolPlan::default(),
    );

    let fragment = &members_for_class(&contribution, "Graph")[0];
    assert!(fragment.contains("List<Node>.of("));
    assert!(fragment.contains(".map((item_0) => item_0.copyWith())"));
    assert!(!fragment.contains("nextNodesSource"));
    let _ = span(0, 0);
}
