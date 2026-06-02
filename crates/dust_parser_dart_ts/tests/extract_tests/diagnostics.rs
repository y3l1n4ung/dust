use crate::support::parse;

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
