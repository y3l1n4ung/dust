//! Integration tests for resolver symbol ownership and annotation resolution.

use dust_ir::{AnnotationValueIr, SymbolId};
use dust_parser_dart::{ParseBackend, ParseOptions};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::{SymbolCatalog, SymbolKind, resolve_library, validate_generated_part_uri};
use dust_text::{FileId, SourceText};

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
        resolved.library.classes[0].configs[0]
            .arguments_source
            .as_deref(),
        Some("(renameAll: SerDeRename.snakeCase)")
    );
    assert_named_expression(
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
    assert_named_expression(
        &resolved.library.classes[0].fields[0].configs[0],
        "rename",
        "'full_name'",
    );
}

fn assert_named_expression(
    config: &dust_ir::ConfigApplicationIr,
    name: &str,
    expected_source: &str,
) {
    let Some(AnnotationValueIr::Expression(source)) = config.named_argument_value(name) else {
        panic!("expected named expression argument `{name}` in {config:?}");
    };
    assert_eq!(source.source, expected_source);
}

#[test]
fn resolves_validate_as_derive_trait_and_field_config() {
    let source = SourceText::new(
        FileId::new(5),
        r#"
part 'signup.g.dart';

@Derive([Validate()])
class Signup {
  @Validate(email: true)
  final String email;

  const Signup(this.email);
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("Validate", "dust_dart::Validate");
    catalog.register_config("Validate", "dust_dart::Validate");

    let resolved = resolve_library(
        FileId::new(5),
        "lib/signup.dart",
        "lib/signup.g.dart",
        &parsed.library,
        &catalog,
    );

    assert_eq!(resolved.diagnostics, vec![]);
    assert_eq!(resolved.library.classes[0].traits.len(), 1);
    assert_eq!(resolved.library.classes[0].configs.len(), 0);
    assert_eq!(resolved.library.classes[0].fields[0].configs.len(), 1);
    assert_eq!(
        resolved.library.classes[0].fields[0].configs[0].symbol,
        SymbolId::new("dust_dart::Validate")
    );
}

#[test]
fn resolves_prefixed_annotations_by_short_name() {
    let source = SourceText::new(
        FileId::new(6),
        r#"
import 'package:dust_dart/derive.dart' as d;
import 'package:other/derive.dart' as other;

part 'user.g.dart';

@other.Derive([d.ToString()])
@d.SerDe(renameAll: d.SerDeRename.snakeCase)
class User {
  @other.SerDe(rename: 'full_name')
  final String name;

  const User(this.name);
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("ToString", "dust_dart::ToString");
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(6),
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
    assert_eq!(resolved.library.classes[0].traits.len(), 1);
    assert_eq!(
        resolved.library.classes[0].traits[0].symbol,
        SymbolId::new("dust_dart::ToString")
    );
    assert_eq!(resolved.library.classes[0].configs.len(), 1);
    assert_eq!(
        resolved.library.classes[0].configs[0].symbol,
        SymbolId::new("dust_dart::SerDe")
    );
    assert_eq!(resolved.library.classes[0].fields[0].configs.len(), 1);
    assert_eq!(
        resolved.library.classes[0].fields[0].configs[0].symbol,
        SymbolId::new("dust_dart::SerDe")
    );
}

#[test]
fn missing_generated_part_is_reported_when_dust_symbols_are_present() {
    let source = SourceText::new(
        FileId::new(2),
        r#"
@Derive([ToString()])
class User {
  final String name;
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("ToString", "dust_dart::ToString");

    let resolved = resolve_library(
        FileId::new(2),
        "lib/user.dart",
        "lib/user.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(!resolved.diagnostics.is_empty());
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("missing generated `part` directive")
    }));
}

#[test]
fn unknown_derive_members_are_reported_but_do_not_abort_resolution() {
    let source = SourceText::new(
        FileId::new(3),
        r#"
part 'user.g.dart';

@Derive([ToString(), UnknownThing()])
class User {}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("ToString", "dust_dart::ToString");

    let resolved = resolve_library(
        FileId::new(3),
        "lib/user.dart",
        "lib/user.g.dart",
        &parsed.library,
        &catalog,
    );

    assert_eq!(resolved.library.classes[0].traits.len(), 1);
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unknown derive trait or config")
    }));
}

#[test]
fn field_trait_annotations_are_reported_as_invalid() {
    let source = SourceText::new(
        FileId::new(4),
        r#"
part 'user.g.dart';

class User {
  @Serialize()
  final String name;
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("Serialize", "dust_dart::Serialize");

    let resolved = resolve_library(
        FileId::new(4),
        "lib/user.dart",
        "lib/user.g.dart",
        &parsed.library,
        &catalog,
    );

    assert_eq!(resolved.library.classes[0].fields.len(), 1);
    assert!(resolved.library.classes[0].fields[0].configs.is_empty());
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("trait annotation `Serialize` is not supported on fields")
    }));
}
