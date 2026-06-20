//! Integration tests for serde plugin validation diagnostics.

use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, EnumIr, FieldIr, LibraryIr, ParamKind,
    SpanIr, SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::DustPlugin;
use dust_plugin_serde::register_plugin;
use dust_text::{FileId, TextRange};

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
