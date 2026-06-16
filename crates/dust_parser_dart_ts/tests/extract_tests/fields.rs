use dust_parser_dart::ParsedTypeKind;

use crate::support::parse;

#[test]
fn extracts_field_annotations_and_argument_source() {
    let result = parse(
        7,
        r#"
part 'user.g.dart';

class User {
  @SerDe(rename: 'user_id', renameAll: SerDeRename.snakeCase)
  final String id;

  @Deprecated('legacy')
  @SerDe(defaultValue: <String>[])
  final Map<String, List<int?>> tags = const {};

  final int count = fallbackCount;
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    assert_eq!(class.fields.len(), 3);

    let id_field = &class.fields[0];
    assert_eq!(id_field.name, "id");
    assert_eq!(id_field.type_source.as_deref(), Some("String"));
    assert_eq!(
        id_field.parsed_type.as_ref().map(|ty| &ty.kind),
        Some(&ParsedTypeKind::Builtin("String".to_owned()))
    );
    assert_eq!(id_field.annotations.len(), 1);
    assert!(id_field.has_annotation("SerDe"));
    assert_eq!(
        id_field.annotations[0].arguments_source.as_deref(),
        Some("(rename: 'user_id', renameAll: SerDeRename.snakeCase)")
    );
    let id_args = id_field.annotations[0].parsed_arguments.as_ref().unwrap();
    assert!(id_args.positional.is_empty());
    assert_eq!(id_args.named.len(), 2);
    assert_eq!(id_args.named[0].name, "rename");
    assert_eq!(id_args.named[0].source, "rename: 'user_id'");
    assert_eq!(id_args.named[0].value_source, "'user_id'");
    assert_eq!(id_args.named[1].name, "renameAll");
    assert_eq!(id_args.named[1].source, "renameAll: SerDeRename.snakeCase");
    assert_eq!(id_args.named[1].value_source, "SerDeRename.snakeCase");

    let tags_field = &class.fields[1];
    assert_eq!(tags_field.name, "tags");
    assert_eq!(
        tags_field.type_source.as_deref(),
        Some("Map<String, List<int?>>")
    );
    let tags_type = tags_field.parsed_type.as_ref().unwrap();
    assert_eq!(tags_type.source, "Map<String, List<int?>>");
    let ParsedTypeKind::Named { name, args } = &tags_type.kind else {
        panic!("expected named tags type");
    };
    assert_eq!(name, "Map");
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].kind, ParsedTypeKind::Builtin("String".to_owned()));
    let ParsedTypeKind::Named {
        name: list_name,
        args: list_args,
    } = &args[1].kind
    else {
        panic!("expected nested list type");
    };
    assert_eq!(list_name, "List");
    assert_eq!(list_args.len(), 1);
    assert_eq!(list_args[0].kind, ParsedTypeKind::Builtin("int".to_owned()));
    assert!(list_args[0].nullable);
    assert!(tags_field.has_default);
    assert_eq!(tags_field.annotations.len(), 2);
    assert_eq!(tags_field.annotations[0].name, "Deprecated");
    assert_eq!(
        tags_field.annotations[0].arguments_source.as_deref(),
        Some("('legacy')")
    );
    let deprecated_args = tags_field.annotations[0].parsed_arguments.as_ref().unwrap();
    assert_eq!(deprecated_args.positional.len(), 1);
    assert_eq!(deprecated_args.positional[0].source, "'legacy'");
    assert!(deprecated_args.named.is_empty());
    assert_eq!(tags_field.annotations[1].name, "SerDe");
    assert_eq!(
        tags_field.annotations[1].arguments_source.as_deref(),
        Some("(defaultValue: <String>[])")
    );
    let serde_args = tags_field.annotations[1].parsed_arguments.as_ref().unwrap();
    assert!(serde_args.positional.is_empty());
    assert_eq!(serde_args.named.len(), 1);
    assert_eq!(serde_args.named[0].name, "defaultValue");
    assert_eq!(serde_args.named[0].value_source, "<String>[]");

    let count_field = &class.fields[2];
    assert_eq!(count_field.name, "count");
    assert_eq!(count_field.type_source.as_deref(), Some("int"));
    assert!(count_field.has_default);
}
