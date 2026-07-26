use crate::support::parse;

fn modifier_diagnostic_count(result: &dust_parser_dart::ParseResult, keyword: &str) -> usize {
    let message = format!("`{keyword}` is only valid on primary-constructor declaring parameters");
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains(&message))
        .count()
}

#[test]
fn reports_diagnostic_for_malformed_dart_source() {
    let result = parse(
        3,
        r#"
class Broken {
  final String name
}
"#,
    );

    assert!(result.has_errors());
    assert!(!result.diagnostics.is_empty());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("syntax error"))
    );
}

#[test]
fn reports_invalid_parameter_modifiers_for_dart_3_13_normal_params() {
    let result = parse(
        310,
        r#"
// @dart=3.13
void saveFinal(final String name) {}
void saveVar(var String name) {}

class User {
  const User(final String id);

  void update({
    required final String name,
    covariant var int count,
  }) {}
}
"#,
    );

    assert!(result.has_errors());
    assert_eq!(modifier_diagnostic_count(&result, "final"), 3);
    assert_eq!(modifier_diagnostic_count(&result, "var"), 2);
    assert!(
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .notes
                .iter()
                .any(|note| note.contains("Use `Type name` for normal function"))
        }),
        "diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn preserves_normal_parameter_modifiers_before_dart_3_13() {
    let result = parse(
        311,
        r#"
// @dart=3.12
void saveFinal(final String name) {}
void saveVar(var String name) {}

class User {
  const User(final String id);
  void update({required final String name, var int count = 0}) {}
}
"#,
    );

    assert_eq!(modifier_diagnostic_count(&result, "final"), 0);
    assert_eq!(modifier_diagnostic_count(&result, "var"), 0);
}

#[test]
fn ignores_existing_field_and_super_formals() {
    let result = parse(
        312,
        r#"
// @dart=3.13
class Parent {
  const Parent({Object? key});
}

class User extends Parent {
  const User(this.id, {required this.name, super.key});

  final String id;
  final String name;
}
"#,
    );

    assert_eq!(modifier_diagnostic_count(&result, "final"), 0);
    assert_eq!(modifier_diagnostic_count(&result, "var"), 0);
}
