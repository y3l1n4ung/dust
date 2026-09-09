//! Serde configuration on enum variants and constructor-declared variants.

use super::*;

#[test]
fn resolves_enum_variant_configs() {
    let source = SourceText::new(
        FileId::new(9),
        r#"
part 'status.g.dart';

@SerDe(renameAll: SerDeRename.snakeCase)
enum Status {
  @SerDe(rename: 'pending')
  pendingReview,
  @SerDe(skip: true)
  legacyFailed,
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(9),
        "lib/status.dart",
        "lib/status.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let enum_ = &resolved.library.enums[0];
    assert_eq!(enum_.configs.len(), 1);
    assert_eq!(enum_.variants.len(), 2);
    assert_eq!(enum_.variants[0].configs.len(), 1);
    assert_eq!(
        enum_.variants[0]
            .serde
            .as_ref()
            .and_then(|serde| serde.rename.as_deref()),
        Some("pending")
    );
    assert_named_string(&enum_.variants[0].configs[0], "rename", "pending");
    assert_eq!(enum_.variants[1].configs.len(), 1);
    assert!(
        enum_.variants[1]
            .serde
            .as_ref()
            .is_some_and(|serde| serde.skip)
    );
    assert_named_bool(&enum_.variants[1].configs[0], "skip", true);
}

#[test]
fn constructor_configs_require_generated_part_directive() {
    let source = SourceText::new(
        FileId::new(8),
        r#"
sealed class AuthEvent {
  const AuthEvent();

  @SerDe(rename: 'login')
  factory AuthEvent.userLoggedIn() = UserLoggedIn;
}

final class UserLoggedIn extends AuthEvent {
  const UserLoggedIn() : super();
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(8),
        "lib/auth_event.dart",
        "lib/auth_event.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("missing generated `part` directive")
    }));
}
