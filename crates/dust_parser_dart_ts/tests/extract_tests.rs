use dust_parser_dart::{ParseBackend, ParseOptions, ParsedClassKind, ParsedDirective};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_text::{FileId, SourceText};

#[test]
fn extracts_library_surface_from_real_dart_source() {
    let source = SourceText::new(
        FileId::new(1),
        r#"
import 'dart:convert';
part 'user.g.dart';

@Derive([Debug(), Serialize(), Deserialize()])
class User {
  final String name;
  final int? age;

  const User(this.name, this.age);
}
"#,
    );

    let result = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());

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
fn extracts_named_constructor_and_named_parameters() {
    let source = SourceText::new(
        FileId::new(2),
        r#"
part 'user_profile.g.dart';

class UserProfile<T> {
  final List<T> items;

  const UserProfile.named({required this.items});
}
"#,
    );

    let result = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    assert_eq!(class.kind, ParsedClassKind::Class);
    assert_eq!(class.name, "UserProfile");
    assert_eq!(class.fields.len(), 1);
    assert_eq!(class.fields[0].type_source.as_deref(), Some("List<T>"));
    assert_eq!(class.constructors.len(), 1);
    assert_eq!(class.constructors[0].name.as_deref(), Some("named"));
    assert_eq!(class.constructors[0].params.len(), 1);
    assert_eq!(class.constructors[0].params[0].name, "items");
    assert_eq!(
        class.constructors[0].params[0].kind,
        dust_parser_dart::ParameterKind::Named
    );
}

#[test]
fn extracts_field_annotations_and_argument_source() {
    let source = SourceText::new(
        FileId::new(7),
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

    let result = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());

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

#[test]
fn reports_diagnostic_for_malformed_dart_source() {
    let source = SourceText::new(
        FileId::new(3),
        r#"
class Broken {
  final String name
}
"#,
    );

    let result = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());

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
fn extracts_abstract_and_extends_class_shapes() {
    let source = SourceText::new(
        FileId::new(4),
        r#"
part 'entity.g.dart';

abstract class Entity extends CatalogNode with AuditStamp {
  final String id;

  const Entity(this.id);
}

class ProductNode extends CatalogNode with AuditStamp {
  final String sku;

  const ProductNode(this.sku);
}
"#,
    );

    let result = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.library.classes.len(), 2);
    assert!(result.library.classes[0].is_abstract);
    assert_eq!(
        result.library.classes[0].superclass_name.as_deref(),
        Some("CatalogNode")
    );
    assert_eq!(result.library.classes[0].name, "Entity");
    assert_eq!(result.library.classes[0].fields.len(), 1);
    assert_eq!(result.library.classes[0].fields[0].name, "id");
    assert!(!result.library.classes[1].is_abstract);
    assert_eq!(
        result.library.classes[1].superclass_name.as_deref(),
        Some("CatalogNode")
    );
    assert_eq!(result.library.classes[1].name, "ProductNode");
    assert_eq!(result.library.classes[1].fields.len(), 1);
    assert_eq!(result.library.classes[1].fields[0].name, "sku");
}

#[test]
fn extracts_classes_with_mixin_clauses_and_ignores_plain_mixin_declarations() {
    let source = SourceText::new(
        FileId::new(5),
        r#"
part 'tagged_value.g.dart';

mixin AuditStamp {
  String auditLabel() => 'audited';
}

class TaggedValue with AuditStamp {
  final String code;
  final List<String> aliases;

  const TaggedValue(this.code, this.aliases);
}

class TaggedSku extends TaggedValue with AuditStamp {
  const TaggedSku(super.code, super.aliases);
}
"#,
    );

    let result = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.library.classes.len(), 2);
    assert_eq!(result.library.classes[0].kind, ParsedClassKind::Class);
    assert_eq!(result.library.classes[0].name, "TaggedValue");
    assert_eq!(result.library.classes[0].fields.len(), 2);
    assert_eq!(
        result.library.classes[0].fields[1].type_source.as_deref(),
        Some("List<String>")
    );
    assert_eq!(result.library.classes[0].constructors.len(), 1);
    assert_eq!(result.library.classes[1].name, "TaggedSku");
    assert_eq!(result.library.classes[1].constructors.len(), 1);
}

#[test]
fn detects_mixin_class_declarations_for_validation() {
    let source = SourceText::new(
        FileId::new(6),
        r#"
part 'mixin_target.g.dart';

@Derive([Debug()])
abstract mixin class MixinTarget {
  const MixinTarget(this.id);

  final String id;
}
"#,
    );

    let result = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.library.classes.len(), 1);
    assert_eq!(result.library.classes[0].kind, ParsedClassKind::MixinClass);
    assert!(result.library.classes[0].is_abstract);
}
