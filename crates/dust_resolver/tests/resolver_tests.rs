//! Integration tests for resolver symbol ownership and annotation resolution.

use dust_ir::{AnnotationValueIr, ClassKindIr, SerdeRenameRuleIr, SymbolId};
use dust_parser_dart::{ParseBackend, ParseOptions, ParsedAnnotation};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::{
    SymbolCatalog, SymbolKind, resolve_annotation_ir, resolve_library, validate_generated_part_uri,
};
use dust_text::{FileId, SourceText, TextRange};

/// Annotation prefixes, generated part directives and resolution diagnostics.
#[path = "resolver_tests/prefixes.rs"]
mod prefixes;

/// Serde normalization across classes, sealed hierarchies and enum variants.
#[path = "resolver_tests/serde.rs"]
mod serde;

/// Serde configuration on enum and constructor-declared variants.
#[path = "resolver_tests/serde_variants.rs"]
mod serde_variants;

#[test]
fn symbol_catalog_registers_traits_and_configs() {
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("ToString", "dust_dart::ToString");
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let to_string = catalog.resolve("ToString").unwrap();
    let serde = catalog.resolve("SerDe").unwrap();

    assert_eq!(to_string.symbol, SymbolId::new("dust_dart::ToString"));
    assert_eq!(to_string.kind, SymbolKind::Trait);
    assert_eq!(serde.kind, SymbolKind::Config);
}

#[test]
fn symbol_catalog_supports_same_surface_name_for_trait_and_config() {
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("Validate", "dust_dart::Validate");
    catalog.register_config("Validate", "dust_dart::Validate");

    assert_eq!(
        catalog.resolve_trait("Validate").unwrap().symbol,
        SymbolId::new("dust_dart::Validate")
    );
    assert_eq!(
        catalog.resolve_config("Validate").unwrap().kind,
        SymbolKind::Config
    );
}

#[test]
fn resolves_registered_annotation_into_canonical_ir() {
    let annotation = ParsedAnnotation {
        name: "SerDe".to_owned(),
        prefix: None,
        qualified_name: "SerDe".to_owned(),
        arguments_source: None,
        parsed_arguments: None,
        span: TextRange::new(0_u32, 6_u32),
    };
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let annotation = resolve_annotation_ir(FileId::new(2), &annotation, &catalog);

    assert_eq!(
        annotation.resolved_symbol,
        Some(SymbolId::new("dust_dart::SerDe"))
    );
}

#[test]
fn resolves_registered_annotation_by_canonical_symbol_name() {
    let annotation = ParsedAnnotation {
        name: "SerDe".to_owned(),
        prefix: Some("dust_dart".to_owned()),
        qualified_name: "dust_dart::SerDe".to_owned(),
        arguments_source: None,
        parsed_arguments: None,
        span: TextRange::new(0_u32, 6_u32),
    };
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_annotation_ir(FileId::new(3), &annotation, &catalog);

    assert_eq!(
        resolved.resolved_symbol,
        Some(SymbolId::new("dust_dart::SerDe"))
    );
}

#[test]
fn validate_generated_part_uri_rejects_wrong_file_name() {
    let diagnostic = validate_generated_part_uri("lib/user.g.dart", "team.g.dart").unwrap_err();

    assert!(
        diagnostic
            .message
            .contains("does not match expected `user.g.dart`")
    );
}

#[test]
fn resolves_real_dart_traits_and_configs() {
    let source = SourceText::new(
        FileId::new(1),
        r#"
part 'user.g.dart';

@Derive([ToString(), Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
class User {
  @SerDe(rename: 'full_name')
  final String name;

  const User(this.name);
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("ToString", "dust_dart::ToString");
    catalog.register_trait("Serialize", "dust_dart::Serialize");
    catalog.register_trait("Deserialize", "dust_dart::Deserialize");
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(1),
        "lib/user.dart",
        "lib/user.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    assert_eq!(resolved.library.output_path, "lib/user.g.dart");
    assert_eq!(resolved.library.part_uri.as_deref(), Some("user.g.dart"));
    assert_eq!(resolved.library.classes.len(), 1);
    assert_eq!(resolved.library.classes[0].traits.len(), 3);
    assert_eq!(resolved.library.classes[0].configs.len(), 1);
    assert_eq!(
        resolved.library.classes[0]
            .serde
            .as_ref()
            .and_then(|serde| serde.rename_all),
        Some(SerdeRenameRuleIr::SnakeCase)
    );
    assert_eq!(
        resolved.library.classes[0].configs[0]
            .arguments_source
            .as_deref(),
        Some("(renameAll: SerDeRename.snakeCase)")
    );
    assert_named_member(
        &resolved.library.classes[0].configs[0],
        "renameAll",
        "SerDeRename.snakeCase",
    );
    assert_eq!(resolved.library.classes[0].fields.len(), 1);
    assert_eq!(resolved.library.classes[0].fields[0].configs.len(), 1);
    assert_eq!(
        resolved.library.classes[0].fields[0].configs[0]
            .arguments_source
            .as_deref(),
        Some("(rename: 'full_name')")
    );
    assert_named_string(
        &resolved.library.classes[0].fields[0].configs[0],
        "rename",
        "full_name",
    );
}

fn assert_named_member(config: &dust_ir::ConfigApplicationIr, name: &str, expected_source: &str) {
    let Some(AnnotationValueIr::Member(source)) = config.named_argument_value(name) else {
        panic!("expected named member argument `{name}` in {config:?}");
    };
    assert_eq!(source.source, expected_source);
}

fn assert_named_string(config: &dust_ir::ConfigApplicationIr, name: &str, expected: &str) {
    let Some(AnnotationValueIr::String(value)) = config.named_argument_value(name) else {
        panic!("expected named string argument `{name}` in {config:?}");
    };
    assert_eq!(value, expected);
}

fn assert_named_bool(config: &dust_ir::ConfigApplicationIr, name: &str, expected: bool) {
    let Some(AnnotationValueIr::Bool(value)) = config.named_argument_value(name) else {
        panic!("expected named bool argument `{name}` in {config:?}");
    };
    assert_eq!(*value, expected);
}
