use crate::support::parse;

#[test]
fn extracts_method_modifiers_and_bodies_from_tree_sitter_nodes() {
    let result = parse(
        22,
        r#"
class MethodShapes {
  external static Future<void> load();
  static int answer() => 42;
  String describe() { return 'ok'; }
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    assert_eq!(class.methods.len(), 3);

    let external_static = &class.methods[0];
    assert_eq!(external_static.name, "load");
    assert!(external_static.is_external);
    assert!(external_static.is_static);
    assert!(!external_static.has_body);
    assert_eq!(external_static.body_source, None);
    assert_eq!(
        external_static.return_type_source.as_deref(),
        Some("Future<void>")
    );

    let expression_body = &class.methods[1];
    assert_eq!(expression_body.name, "answer");
    assert!(expression_body.is_static);
    assert!(!expression_body.is_external);
    assert!(expression_body.has_body);
    assert_eq!(expression_body.body_source.as_deref(), Some("=> 42;"));
    assert_eq!(expression_body.return_type_source.as_deref(), Some("int"));

    let block_body = &class.methods[2];
    assert_eq!(block_body.name, "describe");
    assert!(!block_body.is_static);
    assert!(!block_body.is_external);
    assert!(block_body.has_body);
    assert_eq!(block_body.body_source.as_deref(), Some("{ return 'ok'; }"));
    assert_eq!(block_body.return_type_source.as_deref(), Some("String"));
}
