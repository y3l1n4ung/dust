//! Integration smoke tests for shared Dart emission facade helpers.

use dust_dart_emit::{
    DART_MAP, OBJECT_NULLABLE_TYPES, apply_rename_rule, dart_string_literal, non_nullable,
    parse_named_arguments, parse_static_dart_string_literal, render_template,
};
use dust_ir::{SerdeRenameRuleIr, TypeIr};

#[derive(serde::Serialize)]
struct HelperContext<'a> {
    class_name: &'a str,
    wire_key: String,
    dart_type: String,
    literal: String,
}

#[test]
fn public_facade_helpers_compose_plugin_style_output() {
    let field_type = TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::named("Todo")));
    let nullable_field_type = field_type.clone().nullable();
    let context = HelperContext {
        class_name: "TodoResponse",
        wire_key: apply_rename_rule("createdAt", SerdeRenameRuleIr::SnakeCase),
        dart_type: OBJECT_NULLABLE_TYPES.render(&nullable_field_type),
        literal: dart_string_literal("can't interpolate $name"),
    };

    let rendered = render_template(
        "field",
        r#"class {{ class_name }} {
  final {{ dart_type }} {{ wire_key }};
  static const sample = {{ literal }};
}
"#,
        context,
    );

    assert_eq!(
        rendered,
        r#"class TodoResponse {
  final Map<String, List<Todo>>? created_at;
  static const sample = 'can\'t interpolate \$name';
}"#
    );
    assert_eq!(DART_MAP, "Map");
    assert_eq!(non_nullable(&nullable_field_type), field_type);
}

#[test]
fn public_facade_reexports_source_scanners_used_by_emitters() {
    let args = parse_named_arguments(Some(
        "(rename: 'wire_name', codec: const JsonCodec<Map<String, Object?>>())",
    ));

    assert_eq!(
        args,
        vec![
            ("rename", "'wire_name'"),
            ("codec", "const JsonCodec<Map<String, Object?>>()"),
        ]
    );
    assert_eq!(
        parse_static_dart_string_literal(r"r'''SELECT * FROM todos WHERE id = $1'''"),
        Some("SELECT * FROM todos WHERE id = $1".to_owned())
    );
}
