//! Integration tests for resolver symbol ownership and annotation resolution.

use dust_ir::{AnnotationValueIr, ClassKindIr, SerdeRenameRuleIr, SymbolId};
use dust_parser_dart::{ParseBackend, ParseOptions, ParsedAnnotation};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_resolver::{
    SymbolCatalog, SymbolKind, resolve_annotation_ir, resolve_library, validate_generated_part_uri,
};
use dust_text::{FileId, SourceText, TextRange};

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

#[test]
fn normalizes_serde_from_structured_values_without_rescanning_raw_source() {
    let source = SourceText::new(
        FileId::new(7),
        r#"
part 'user.g.dart';

@SerDe(renameAll: SerDeRename.snakeCase)
class User {
  @SerDe(rename: 'full_name')
  final String name;

  const User(this.name);
}
"#,
    );

    let mut parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    parsed.library.classes[0].annotations[0].arguments_source = Some("not valid".to_owned());
    parsed.library.classes[0].fields[0].annotations[0].arguments_source =
        Some("also not valid".to_owned());

    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(7),
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
    assert_eq!(
        resolved.library.classes[0]
            .serde
            .as_ref()
            .and_then(|serde| serde.rename_all),
        Some(SerdeRenameRuleIr::SnakeCase)
    );
    assert_eq!(
        resolved.library.classes[0].fields[0]
            .serde
            .as_ref()
            .and_then(|serde| serde.rename.as_deref()),
        Some("full_name")
    );
}

#[test]
fn resolves_constructor_configs_for_sealed_factory_variants() {
    let source = SourceText::new(
        FileId::new(7),
        r#"
part 'auth_event.g.dart';

@SerDe(tag: 'type')
sealed class AuthEvent {
  const AuthEvent();

  @SerDe(rename: 'login')
  factory AuthEvent.userLoggedIn({required String userId}) = UserLoggedIn;
}

final class UserLoggedIn extends AuthEvent {
  const UserLoggedIn({required this.userId}) : super();

  final String userId;
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(7),
        "lib/auth_event.dart",
        "lib/auth_event.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let class = &resolved.library.classes[0];
    assert_eq!(class.kind, ClassKindIr::SealedClass);
    assert_eq!(class.configs.len(), 1);
    assert_eq!(class.constructors.len(), 2);
    let constructor = class
        .constructors
        .iter()
        .find(|constructor| constructor.surface.name.as_deref() == Some("userLoggedIn"))
        .expect("expected `AuthEvent.userLoggedIn` constructor");
    assert_eq!(constructor.configs.len(), 1);
    assert_eq!(
        constructor.configs[0].symbol,
        SymbolId::new("dust_dart::SerDe")
    );
    assert_eq!(
        constructor.configs[0].arguments_source.as_deref(),
        Some("(rename: 'login')")
    );
    assert_named_string(&constructor.configs[0], "rename", "login");
    let serde = class.serde.as_ref().expect("sealed class serde");
    assert_eq!(serde.tag.as_deref(), Some("type"));
    assert_eq!(serde.variants.len(), 1);
    assert_eq!(serde.variants[0].constructor_name, "userLoggedIn");
    assert_eq!(serde.variants[0].target_class_name, "UserLoggedIn");
    assert_eq!(serde.variants[0].tag, "login");
    assert_eq!(serde.variants[0].params.len(), 1);
    assert_eq!(serde.variants[0].params[0].name, "userId");
    assert!(serde.variants[0].params[0].ty.is_named("String"));
}

#[test]
fn resolves_sealed_serde_variants_with_rename_all_and_missing_target_class() {
    let source = SourceText::new(
        FileId::new(10),
        r#"
part 'payment_event.g.dart';

@SerDe(tag: 'type', renameAll: SerDeRename.snakeCase)
sealed class PaymentEvent {
  const PaymentEvent();

  factory PaymentEvent.paymentSucceeded({required String id, required int cents}) =
      PaymentSucceeded;
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(10),
        "lib/payment_event.dart",
        "lib/payment_event.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    let serde = resolved.library.classes[0].serde.as_ref().unwrap();
    assert_eq!(serde.variants.len(), 1);
    assert_eq!(serde.variants[0].target_class_name, "PaymentSucceeded");
    assert_eq!(serde.variants[0].tag, "payment_succeeded");
    assert_eq!(serde.variants[0].params.len(), 2);
    assert!(serde.variants[0].params[0].ty.is_named("String"));
    assert!(serde.variants[0].params[1].ty.is_named("int"));
}

#[test]
fn reports_invalid_sealed_serde_variant_metadata() {
    let source = SourceText::new(
        FileId::new(11),
        r#"
part 'events.g.dart';

@SerDe(tag: 'type')
class NotSealed {}

@SerDe(tag: 'type')
sealed class EmptyEvent {
  const EmptyEvent();
}

@SerDe(tag: 'type')
sealed class DuplicateEvent {
  const DuplicateEvent();

  @SerDe(rename: 'same')
  factory DuplicateEvent.first() = FirstEvent;

  @SerDe(rename: 'same')
  factory DuplicateEvent.second() = SecondEvent;
}

final class FirstEvent extends DuplicateEvent {
  const FirstEvent() : super();
}

final class SecondEvent extends DuplicateEvent {
  const SecondEvent() : super();
}

@SerDe(tag: 'type')
sealed class BadTargetEvent {
  const BadTargetEvent();

  factory BadTargetEvent.bad() = WrongBaseEvent;
}

final class WrongBaseEvent extends DuplicateEvent {
  const WrongBaseEvent() : super();
}
"#,
    );

    let parsed = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut catalog = SymbolCatalog::new();
    catalog.register_config("SerDe", "dust_dart::SerDe");

    let resolved = resolve_library(
        FileId::new(11),
        "lib/events.dart",
        "lib/events.g.dart",
        &parsed.library,
        &catalog,
    );

    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("SerDe class `NotSealed` uses sealed variant options but is not sealed")
    }));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Sealed SerDe class EmptyEvent has no factory variants")
    }));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Duplicate SerDe variant tag: same")
    }));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Variant target class WrongBaseEvent does not extend BadTargetEvent")
    }));
}

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
