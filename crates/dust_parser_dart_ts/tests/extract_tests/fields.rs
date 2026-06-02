use crate::support::parse;

#[test]
fn extracts_field_annotations_and_argument_source() {
    let result = parse(
        7,
        r#"
part 'user.g.dart';

class User {
  @SerDe(rename: 'user_id')
  final String id;

  @Deprecated('legacy')
  @SerDe(defaultValue: <String>[])
  final List<String> tags = const [];
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    assert_eq!(class.fields.len(), 2);

    let id_field = &class.fields[0];
    assert_eq!(id_field.name, "id");
    assert_eq!(id_field.type_source.as_deref(), Some("String"));
    assert_eq!(id_field.annotations.len(), 1);
    assert!(id_field.has_annotation("SerDe"));
    assert_eq!(
        id_field.annotations[0].arguments_source.as_deref(),
        Some("(rename: 'user_id')")
    );

    let tags_field = &class.fields[1];
    assert_eq!(tags_field.name, "tags");
    assert_eq!(tags_field.type_source.as_deref(), Some("List<String>"));
    assert!(tags_field.has_default);
    assert_eq!(tags_field.annotations.len(), 2);
    assert_eq!(tags_field.annotations[0].name, "Deprecated");
    assert_eq!(
        tags_field.annotations[0].arguments_source.as_deref(),
        Some("('legacy')")
    );
    assert_eq!(tags_field.annotations[1].name, "SerDe");
    assert_eq!(
        tags_field.annotations[1].arguments_source.as_deref(),
        Some("(defaultValue: <String>[])")
    );
}
