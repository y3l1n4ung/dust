//! Annotation prefixes, generated part directives and resolution diagnostics.

use super::*;

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
fn reports_annotation_prefixes_without_matching_imports() {
    let source = SourceText::new(
        FileId::new(8),
        r#"
part 'user.g.dart';

@dust.SerDe(rename: 'user')
class User {}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(8),
        "lib/user.dart",
        "lib/user.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("annotation prefix `dust` is not declared by an import")
    }));
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
