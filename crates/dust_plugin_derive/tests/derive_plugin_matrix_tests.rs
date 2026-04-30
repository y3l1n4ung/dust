use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind,
    SpanIr, SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(7), TextRange::new(start, end))
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

fn library(classes: Vec<ClassIr>) -> LibraryIr {
    LibraryIr {
        source_path: "lib/models.dart".to_owned(),
        output_path: "lib/models.g.dart".to_owned(),
        span: span(0, 200),
        classes,
        enums: Vec::new(),
    }
}

fn members_for_class<'a>(
    contribution: &'a dust_plugin_api::PluginContribution,
    class_name: &str,
) -> &'a [String] {
    contribution
        .mixin_members
        .iter()
        .find(|entry| entry.class_name == class_name)
        .map(|entry| entry.members.as_slice())
        .unwrap_or(&[])
}

#[test]
fn emits_debug_eq_and_hash_for_zero_field_class() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Unit",
            Vec::new(),
            vec![constructor(None, Vec::new())],
            &["derive_annotation::ToString", "derive_annotation::Eq"],
        )]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Unit");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(members.len(), 3);
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("String toString() {\n  return 'Unit()';\n}"))
    );
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("other is Unit &&"))
    );
    assert!(members.iter().any(|fragment| {
        fragment.contains("int get hashCode => Object.hashAll([\n  runtimeType,\n]);")
    }));
}

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
fn validation_accumulates_multiple_class_errors() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library(vec![class(
        "BrokenCopyWith",
        vec![field("id", TypeIr::string()), field("age", TypeIr::int())],
        vec![constructor(
            None,
            vec![constructor_param(
                "id",
                TypeIr::string(),
                ParamKind::Positional,
            )],
        )],
        &["derive_annotation::CopyWith"],
    )]));

    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic
                .message
                .contains("`CopyWith` requires a constructor that accepts every field on class `BrokenCopyWith`"))
    );
}

#[test]
fn requested_symbols_are_deduped_for_multiple_copywith_classes() {
    let plugin = register_plugin();
    let requested = plugin.requested_symbols(&library(vec![
        class(
            "User",
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
            "Team",
            vec![field("name", TypeIr::string())],
            vec![constructor(
                None,
                vec![constructor_param(
                    "name",
                    TypeIr::string(),
                    ParamKind::Positional,
                )],
            )],
            &["derive_annotation::CopyWith"],
        ),
    ]));

    assert!(requested.is_empty());
}

#[test]
fn emits_fragments_for_multiple_classes_in_stable_feature_order() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![
            class(
                "User",
                vec![field("id", TypeIr::string())],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "id",
                        TypeIr::string(),
                        ParamKind::Positional,
                    )],
                )],
                &["derive_annotation::ToString", "derive_annotation::Eq"],
            ),
            class(
                "Team",
                vec![field("name", TypeIr::string())],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "name",
                        TypeIr::string(),
                        ParamKind::Positional,
                    )],
                )],
                &["derive_annotation::CopyWith"],
            ),
        ]),
        &SymbolPlan::default(),
    );
    let user_members = members_for_class(&contribution, "User");
    let team_members = members_for_class(&contribution, "Team");

    assert_eq!(contribution.mixin_members.len(), 2);
    assert_eq!(user_members.len(), 3);
    assert_eq!(team_members.len(), 1);
    assert!(user_members[0].contains("String toString() {"));
    assert!(user_members[0].contains("return 'User('"));
    assert!(user_members[0].contains("'id: ${_dustSelf.id}'"));
    assert!(user_members[1].contains("bool operator ==(Object other) =>"));
    assert!(user_members[2].contains("int get hashCode => Object.hashAll(["));
    assert!(team_members[0].contains("Team copyWith({"));
    assert!(team_members[0].contains("String? name,"));
    assert!(team_members[0].contains("name ?? _dustSelf.name,"));
}

#[test]
fn emits_deep_equality_and_hash_for_collection_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Catalog",
            vec![
                field("products", TypeIr::list_of(TypeIr::named("Product"))),
                field(
                    "bySku",
                    TypeIr::map_of(TypeIr::string(), TypeIr::named("Product")),
                ),
                field(
                    "featuredSkus",
                    TypeIr::generic("Set", vec![TypeIr::string()]),
                ),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param(
                        "products",
                        TypeIr::list_of(TypeIr::named("Product")),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "bySku",
                        TypeIr::map_of(TypeIr::string(), TypeIr::named("Product")),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "featuredSkus",
                        TypeIr::generic("Set", vec![TypeIr::string()]),
                        ParamKind::Positional,
                    ),
                ],
            )],
            &["derive_annotation::Eq"],
        )]),
        &SymbolPlan::default(),
    );

    assert!(
        contribution
            .shared_helpers
            .iter()
            .any(|helper| helper.contains("_dustDeepCollectionEquality"))
    );
    assert!(
        contribution
            .shared_helpers
            .iter()
            .any(|helper| helper.contains("_dustUnorderedDeepCollectionEquality"))
    );

    let members = members_for_class(&contribution, "Catalog");
    let eq = members
        .iter()
        .find(|fragment| fragment.contains("bool operator =="))
        .unwrap();
    let hash = members
        .iter()
        .find(|fragment| fragment.contains("int get hashCode"))
        .unwrap();

    assert!(eq.contains("_dustDeepCollectionEquality.equals(other.products, _dustSelf.products)"));
    assert!(eq.contains("_dustDeepCollectionEquality.equals(other.bySku, _dustSelf.bySku)"));
    assert!(eq.contains(
        "_dustUnorderedDeepCollectionEquality.equals(other.featuredSkus, _dustSelf.featuredSkus)"
    ));
    assert!(hash.contains("_dustDeepCollectionEquality.hash(_dustSelf.products)"));
    assert!(hash.contains("_dustDeepCollectionEquality.hash(_dustSelf.bySku)"));
    assert!(hash.contains("_dustUnorderedDeepCollectionEquality.hash(_dustSelf.featuredSkus)"));
}
