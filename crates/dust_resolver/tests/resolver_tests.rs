use dust_ir::SymbolId;
use dust_parser_dart::{ParseBackend, ParseOptions};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::{SymbolCatalog, SymbolKind, resolve_library, validate_generated_part_uri};
use dust_text::{FileId, SourceText};

#[test]
fn symbol_catalog_registers_traits_and_configs() {
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("Debug", "derive_annotation::Debug");
    catalog.register_config("SerDe", "derive_serde_annotation::SerDe");

    let debug = catalog.resolve("Debug").unwrap();
    let serde = catalog.resolve("SerDe").unwrap();

    assert_eq!(debug.symbol, SymbolId::new("derive_annotation::Debug"));
    assert_eq!(debug.kind, SymbolKind::Trait);
    assert_eq!(serde.kind, SymbolKind::Config);
}

#[test]
fn validate_generated_part_uri_rejects_wrong_file_name() {
    let diagnostic = validate_generated_part_uri("lib/user.dart", "team.g.dart").unwrap_err();

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

@Derive([Debug(), Serialize(), Deserialize()])
@SerDe(renameAll: SerdeRename.snakeCase)
class User {
  @SerDe(rename: 'full_name')
  final String name;

  const User(this.name);
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("Debug", "derive_annotation::Debug");
    catalog.register_trait("Serialize", "derive_serde_annotation::Serialize");
    catalog.register_trait("Deserialize", "derive_serde_annotation::Deserialize");
    catalog.register_config("SerDe", "derive_serde_annotation::SerDe");

    let resolved = resolve_library(FileId::new(1), "lib/user.dart", &parsed.library, &catalog);

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
        Some("(renameAll: SerdeRename.snakeCase)")
    );
    assert_eq!(resolved.library.classes[0].fields.len(), 1);
    assert_eq!(resolved.library.classes[0].fields[0].configs.len(), 1);
    assert_eq!(
        resolved.library.classes[0].fields[0].configs[0]
            .arguments_source
            .as_deref(),
        Some("(rename: 'full_name')")
    );
}

#[test]
fn missing_generated_part_is_reported_when_dust_symbols_are_present() {
    let source = SourceText::new(
        FileId::new(2),
        r#"
@Derive([Debug()])
class User {
  final String name;
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("Debug", "derive_annotation::Debug");

    let resolved = resolve_library(FileId::new(2), "lib/user.dart", &parsed.library, &catalog);

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

@Derive([Debug(), UnknownThing()])
class User {}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_trait("Debug", "derive_annotation::Debug");

    let resolved = resolve_library(FileId::new(3), "lib/user.dart", &parsed.library, &catalog);

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
    catalog.register_trait("Serialize", "derive_serde_annotation::Serialize");

    let resolved = resolve_library(FileId::new(4), "lib/user.dart", &parsed.library, &catalog);

    assert_eq!(resolved.library.classes[0].fields.len(), 1);
    assert!(resolved.library.classes[0].fields[0].configs.is_empty());
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("trait annotation `Serialize` is not supported on fields")
    }));
}
