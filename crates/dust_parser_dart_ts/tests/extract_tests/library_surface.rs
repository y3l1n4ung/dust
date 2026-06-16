use dust_parser_dart::{ParsedClassKind, ParsedDirective};

use crate::support::parse;

#[test]
fn extracts_library_surface_from_real_dart_source() {
    let result = parse(
        1,
        r#"
import 'dart:convert';
part 'user.g.dart';

@Derive([ToString(), Serialize(), Deserialize()])
class User {
  final String name;
  final int? age;

  const User(this.name, this.age);
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.library.directives.len(), 2);
    assert_eq!(result.library.classes.len(), 1);
    assert!(!result.library.is_empty());

    match &result.library.directives[0] {
        ParsedDirective::Import { uri, .. } => assert_eq!(uri, "dart:convert"),
        other => panic!("expected import directive, got {other:?}"),
    }

    let class = &result.library.classes[0];
    assert_eq!(class.kind, ParsedClassKind::Class);
    assert_eq!(class.name, "User");
    assert!(!class.is_abstract);
    assert_eq!(class.superclass_name, None);
    assert!(class.has_annotation("Derive"));
    assert_eq!(class.fields.len(), 2);
    assert_eq!(class.fields[0].name, "name");
    assert!(class.fields[0].annotations.is_empty());
    assert_eq!(class.fields[0].type_source.as_deref(), Some("String"));
    assert_eq!(class.fields[1].type_source.as_deref(), Some("int?"));
    assert_eq!(class.constructors.len(), 1);
    assert_eq!(class.constructors[0].name, None);
    assert_eq!(class.constructors[0].params.len(), 2);
}

#[test]
fn extracts_library_and_prefixed_directives() {
    let result = parse(
        1,
        r#"
@Deprecated('legacy')
library models.user;

import 'package:app/src/user.dart' as user;
export 'src/public.dart';
part 'user.g.dart';
part of models.user;
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.library.directives.len(), 5);

    match &result.library.directives[0] {
        ParsedDirective::Library {
            name, annotations, ..
        } => {
            assert_eq!(name.as_deref(), Some("models.user"));
            assert_eq!(annotations.len(), 1);
            assert_eq!(annotations[0].name, "Deprecated");
            assert_eq!(
                annotations[0].positional_argument_source(0),
                Some("'legacy'")
            );
        }
        other => panic!("expected library directive, got {other:?}"),
    }

    match &result.library.directives[1] {
        ParsedDirective::Import { uri, prefix, .. } => {
            assert_eq!(uri, "package:app/src/user.dart");
            assert_eq!(prefix.as_deref(), Some("user"));
        }
        other => panic!("expected import directive, got {other:?}"),
    }

    match &result.library.directives[4] {
        ParsedDirective::PartOf {
            library_name, uri, ..
        } => {
            assert_eq!(library_name.as_deref(), Some("models.user"));
            assert_eq!(uri, &None);
        }
        other => panic!("expected part-of directive, got {other:?}"),
    }
}
