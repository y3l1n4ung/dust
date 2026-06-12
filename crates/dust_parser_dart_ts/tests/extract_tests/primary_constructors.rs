use dust_parser_dart::{ParameterKind, ParsedClassKind};

use crate::support::parse;

#[test]
fn extracts_primary_constructor_declaring_parameters() {
    let result = parse(
        13,
        r#"
class Point(var int x, var int y);

class User({required var String _name});

class DeltaPoint(final int x, int delta) {
  final int y = x + delta;
}
"#,
    );

    assert_eq!(result.diagnostics, vec![]);
    assert_eq!(result.library.classes.len(), 3);

    let point = &result.library.classes[0];
    assert_eq!(point.name, "Point");
    assert_eq!(point.fields.len(), 2);
    assert_eq!(point.fields[0].name, "x");
    assert_eq!(point.fields[0].type_source.as_deref(), Some("int"));
    assert_eq!(point.fields[1].name, "y");
    assert_eq!(point.fields[1].type_source.as_deref(), Some("int"));
    assert_eq!(point.constructors.len(), 1);
    assert_eq!(point.constructors[0].params.len(), 2);
    assert_eq!(point.constructors[0].params[0].name, "x");
    assert_eq!(
        point.constructors[0].params[0].type_source.as_deref(),
        Some("int")
    );
    assert_eq!(
        point.constructors[0].params[0].kind,
        ParameterKind::Positional
    );
    assert_eq!(point.constructors[0].params[1].name, "y");
    assert_eq!(
        point.constructors[0].params[1].type_source.as_deref(),
        Some("int")
    );
    assert_eq!(
        point.constructors[0].params[1].kind,
        ParameterKind::Positional
    );

    let user = &result.library.classes[1];
    assert_eq!(user.name, "User");
    assert_eq!(user.fields.len(), 1);
    assert_eq!(user.fields[0].name, "_name");
    assert_eq!(user.fields[0].type_source.as_deref(), Some("String"));
    assert_eq!(user.constructors.len(), 1);
    assert_eq!(user.constructors[0].params.len(), 1);
    assert_eq!(user.constructors[0].params[0].name, "_name");
    assert_eq!(
        user.constructors[0].params[0].type_source.as_deref(),
        Some("String")
    );
    assert_eq!(user.constructors[0].params[0].kind, ParameterKind::Named);

    let delta = &result.library.classes[2];
    assert_eq!(delta.name, "DeltaPoint");
    assert_eq!(delta.fields.len(), 1);
    assert_eq!(delta.fields[0].name, "x");
    assert_eq!(delta.fields[0].type_source.as_deref(), Some("int"));
    assert_eq!(delta.constructors.len(), 1);
    assert_eq!(delta.constructors[0].params.len(), 2);
    assert_eq!(delta.constructors[0].params[0].name, "x");
    assert_eq!(
        delta.constructors[0].params[0].type_source.as_deref(),
        Some("int")
    );
    assert_eq!(
        delta.constructors[0].params[0].kind,
        ParameterKind::Positional
    );
    assert_eq!(delta.constructors[0].params[1].name, "delta");
    assert_eq!(
        delta.constructors[0].params[1].type_source.as_deref(),
        Some("int")
    );
    assert_eq!(
        delta.constructors[0].params[1].kind,
        ParameterKind::Positional
    );
}

#[test]
fn extracts_primary_constructor_modifiers_annotations_and_defaults() {
    let result = parse(
        14,
        r#"
@Derive([ToString(), Eq(), CopyWith()])
abstract interface class ApiResult<T>({required final String id, var List<T> items = const []});

mixin class Pair(final String left, String label, {final int count = 1});
"#,
    );

    assert_eq!(result.diagnostics, vec![]);
    assert_eq!(result.library.classes.len(), 2);

    let api = &result.library.classes[0];
    assert_eq!(api.kind, ParsedClassKind::Class);
    assert!(api.is_abstract);
    assert!(api.is_interface);
    assert_eq!(api.name, "ApiResult");
    assert_eq!(api.annotations[0].name, "Derive");
    assert_eq!(
        api.annotations[0].arguments_source.as_deref(),
        Some("([ToString(), Eq(), CopyWith()])")
    );
    assert_eq!(api.fields.len(), 2);
    assert_eq!(api.fields[0].name, "id");
    assert_eq!(api.fields[0].type_source.as_deref(), Some("String"));
    assert_eq!(api.fields[1].name, "items");
    assert_eq!(api.fields[1].type_source.as_deref(), Some("List<T>"));
    assert_eq!(api.constructors[0].params[0].kind, ParameterKind::Named);
    assert_eq!(api.constructors[0].params[1].kind, ParameterKind::Named);
    assert_eq!(
        api.constructors[0].params[1]
            .default_value_source
            .as_deref(),
        Some("const []")
    );

    let pair = &result.library.classes[1];
    assert_eq!(pair.kind, ParsedClassKind::MixinClass);
    assert_eq!(pair.fields.len(), 2);
    assert_eq!(pair.fields[0].name, "left");
    assert_eq!(pair.fields[1].name, "count");
    assert_eq!(pair.constructors[0].params.len(), 3);
    assert_eq!(pair.constructors[0].params[1].name, "label");
    assert_eq!(
        pair.constructors[0].params[1].type_source.as_deref(),
        Some("String")
    );
    assert_eq!(
        pair.constructors[0].params[1].kind,
        ParameterKind::Positional
    );
}

#[test]
fn extracts_primary_constructors_without_phantom_classes_or_reordering() {
    let result = parse(
        15,
        r#"
class Ordinary {}

// class Commented(var int value);
const snippet = 'class Quoted(var int value);';

class Primary({var String sep = ',', required var int count});
"#,
    );

    assert_eq!(result.diagnostics, vec![]);
    assert_eq!(result.library.classes.len(), 2);

    let ordinary = &result.library.classes[0];
    assert_eq!(ordinary.name, "Ordinary");
    assert_eq!(ordinary.fields.len(), 0);

    let primary = &result.library.classes[1];
    assert_eq!(primary.name, "Primary");
    assert_eq!(primary.fields.len(), 2);
    assert_eq!(primary.fields[0].name, "sep");
    assert_eq!(primary.fields[0].type_source.as_deref(), Some("String"));
    assert_eq!(primary.fields[1].name, "count");
    assert_eq!(primary.fields[1].type_source.as_deref(), Some("int"));
    assert_eq!(primary.constructors[0].params.len(), 2);
    assert_eq!(primary.constructors[0].params[0].name, "sep");
    assert_eq!(primary.constructors[0].params[0].kind, ParameterKind::Named);
    assert_eq!(
        primary.constructors[0].params[0]
            .default_value_source
            .as_deref(),
        Some("','")
    );
    assert_eq!(primary.constructors[0].params[1].name, "count");
    assert_eq!(primary.constructors[0].params[1].kind, ParameterKind::Named);
}
