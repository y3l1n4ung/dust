//! Serde normalization across classes, sealed hierarchies and enum variants.

use super::*;

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
    let target_class = resolved
        .library
        .classes
        .iter()
        .find(|class| class.name == "UserLoggedIn")
        .expect("expected sealed variant target class");
    assert!(class.requires_lowering_diagnostics);
    assert!(target_class.requires_lowering_diagnostics);
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
