use crate::support::parse;

#[test]
fn extracts_qualified_annotation_names_from_tree_sitter_nodes() {
    let result = parse(
        24,
        r#"
import 'package:dust_dart/derive.dart' as d;

@d.Derive([d.ToString()])
class User {
  @d.SerDe(rename: 'user_id')
  final String id;

  @d.Validate()
  void update(@d.Validate() String value) {}
}

@d.SerDe()
enum Status {
  @d.SerDe(rename: 'active')
  active,
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    let class_annotation = &class.annotations[0];
    assert_eq!(class_annotation.name, "Derive");
    assert_eq!(class_annotation.prefix.as_deref(), Some("d"));
    assert_eq!(class_annotation.qualified_name, "d.Derive");

    let field_annotation = &class.fields[0].annotations[0];
    assert_eq!(field_annotation.name, "SerDe");
    assert_eq!(field_annotation.prefix.as_deref(), Some("d"));
    assert_eq!(field_annotation.qualified_name, "d.SerDe");
    assert_eq!(
        field_annotation.arguments_source.as_deref(),
        Some("(rename: 'user_id')")
    );

    let method_annotation = &class.methods[0].annotations[0];
    assert_eq!(method_annotation.name, "Validate");
    assert_eq!(method_annotation.prefix.as_deref(), Some("d"));
    assert_eq!(method_annotation.qualified_name, "d.Validate");

    let param_annotation = &class.methods[0].params[0].annotations[0];
    assert_eq!(param_annotation.name, "Validate");
    assert_eq!(param_annotation.prefix.as_deref(), Some("d"));
    assert_eq!(param_annotation.qualified_name, "d.Validate");

    let enum_ = &result.library.enums[0];
    let enum_annotation = &enum_.annotations[0];
    assert_eq!(enum_annotation.name, "SerDe");
    assert_eq!(enum_annotation.prefix.as_deref(), Some("d"));
    assert_eq!(enum_annotation.qualified_name, "d.SerDe");

    let variant_annotation = &enum_.variants[0].annotations[0];
    assert_eq!(variant_annotation.name, "SerDe");
    assert_eq!(variant_annotation.prefix.as_deref(), Some("d"));
    assert_eq!(variant_annotation.qualified_name, "d.SerDe");
}
