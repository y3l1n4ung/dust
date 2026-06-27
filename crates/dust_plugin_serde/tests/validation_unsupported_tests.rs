//! Negative validation tests for unsupported SerDe field types.

use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, EnumIr, FieldIr, LibraryIr, ParamKind,
    SpanIr, SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::DustPlugin;
use dust_plugin_serde::register_plugin;
use dust_text::{FileId, TextRange};

#[test]
fn rejects_record_fields_and_generic_named_models() {
    let plugin = register_plugin();
    let record_type = TypeIr::record("({String id})");
    let page_type = TypeIr::generic("Page", vec![TypeIr::named("User")]);
    let target = class(
        "Payload",
        vec![
            field("summary", record_type.clone()),
            field("page", page_type.clone()),
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("summary", record_type, ParamKind::Named),
                constructor_param("page", page_type, ParamKind::Named),
            ],
        )],
        &["dust_dart::Serialize", "dust_dart::Deserialize"],
    );

    let messages = plugin
        .validate(&library(vec![target], vec![]))
        .into_iter()
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "`Serialize` does not support record types on `Payload.summary`",
            "`Deserialize` does not support record types on `Payload.summary`",
            "`Serialize` does not yet support generic named type `Page` on `Payload.page`",
            "`Deserialize` does not yet support generic named type `Page` on `Payload.page`",
        ]
    );
}

/// Builds a source span for validation fixture IR.
fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(11), TextRange::new(start, end))
}

/// Builds a trait application fixture.
fn trait_application(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
        span: span(1, 5),
    }
}

/// Builds a field fixture with default serde settings.
fn field(name: &str, ty: TypeIr) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(10, 20),
        has_default: false,
        serde: None,
        configs: Vec::new(),
    }
}

/// Builds a constructor parameter fixture.
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

/// Builds a generative constructor fixture.
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

/// Builds a class fixture with serde traits.
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

/// Builds a library fixture for validation tests.
fn library(classes: Vec<ClassIr>, enums: Vec<EnumIr>) -> LibraryIr {
    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "dust_test".to_owned(),
        source_path: "lib/models.dart".to_owned(),
        output_path: "lib/models.g.dart".to_owned(),
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(0, 200),
        classes,
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        enums,
        query_calls: Vec::new(),
    }
}
