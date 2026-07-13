use dust_parser_dart::{
    ParsedAnnotationNumberKind, ParsedAnnotationValue, ParsedAnnotationValueRootKind,
};

use crate::support::parse;

#[test]
fn extracts_annotation_value_root_kinds_from_tree_sitter_nodes() {
    let result = parse(
        31,
        r#"
@Meta(
  nothing: null,
  enabled: true,
  name: 'dust',
  count: 1,
  negative: -2,
  ratio: 1.5,
  list: ['a'],
  set: <String>{'a'},
  map: {'a': 1},
  record: (label: 'a'),
  constructed: const Foo.bar(1),
  nested: Length(2, max: 8),
  member: SerDeRename.snakeCase
)
class User {}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let args = result.library.classes[0].annotations[0]
        .parsed_arguments
        .as_ref()
        .expect("expected parsed annotation arguments");

    assert_eq!(kind(args, "nothing"), &ParsedAnnotationValueRootKind::Null);
    assert_eq!(
        kind(args, "enabled"),
        &ParsedAnnotationValueRootKind::Bool(true)
    );
    assert_eq!(source(args, "enabled"), "true");
    assert_eq!(
        kind(args, "name"),
        &ParsedAnnotationValueRootKind::String("dust".to_owned())
    );
    assert_eq!(source(args, "name"), "'dust'");
    assert_eq!(
        kind(args, "count"),
        &ParsedAnnotationValueRootKind::Number(ParsedAnnotationNumberKind::Int)
    );
    assert_eq!(
        kind(args, "negative"),
        &ParsedAnnotationValueRootKind::Number(ParsedAnnotationNumberKind::Int)
    );
    assert_eq!(source(args, "negative"), "-2");
    assert_eq!(
        kind(args, "ratio"),
        &ParsedAnnotationValueRootKind::Number(ParsedAnnotationNumberKind::Double)
    );
    assert!(matches!(
        kind(args, "list"),
        ParsedAnnotationValueRootKind::List(_)
    ));
    let ParsedAnnotationValueRootKind::List(items) = kind(args, "list") else {
        unreachable!();
    };
    assert_eq!(
        items.first().map(|item| &item.kind),
        Some(&ParsedAnnotationValueRootKind::String("a".to_owned()))
    );
    assert_eq!(kind(args, "set"), &ParsedAnnotationValueRootKind::Set);
    assert_eq!(kind(args, "map"), &ParsedAnnotationValueRootKind::Map);
    assert_eq!(kind(args, "record"), &ParsedAnnotationValueRootKind::Record);
    let ParsedAnnotationValueRootKind::Constructor { name, arguments } = kind(args, "constructed")
    else {
        panic!("expected structured constructor");
    };
    assert_eq!(name, "Foo.bar");
    assert_eq!(arguments.positional.len(), 1);
    assert!(arguments.named.is_empty());
    assert_eq!(source(args, "constructed"), "const Foo.bar(1)");
    let ParsedAnnotationValueRootKind::Constructor { name, arguments } = kind(args, "nested")
    else {
        panic!("expected structured nested constructor");
    };
    assert_eq!(name, "Length");
    assert_eq!(arguments.positional.len(), 1);
    assert_eq!(arguments.named.len(), 1);
    assert_eq!(arguments.named[0].name, "max");
    assert_eq!(
        kind(args, "member"),
        &ParsedAnnotationValueRootKind::Member("SerDeRename.snakeCase".to_owned())
    );
    assert_eq!(source(args, "member"), "SerDeRename.snakeCase");
}

fn kind<'a>(
    args: &'a dust_parser_dart::ParsedAnnotationArguments,
    name: &str,
) -> &'a ParsedAnnotationValueRootKind {
    &value(args, name).kind
}

fn source<'a>(args: &'a dust_parser_dart::ParsedAnnotationArguments, name: &str) -> &'a str {
    &value(args, name).source
}

fn value<'a>(
    args: &'a dust_parser_dart::ParsedAnnotationArguments,
    name: &str,
) -> &'a ParsedAnnotationValue {
    args.named
        .iter()
        .find(|arg| arg.name == name)
        .and_then(|arg| arg.value.as_ref())
        .unwrap_or_else(|| panic!("missing typed value for `{name}`"))
}
