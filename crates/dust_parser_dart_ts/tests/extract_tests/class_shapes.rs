use dust_parser_dart::ParsedClassKind;

use crate::support::parse;

#[test]
fn extracts_abstract_and_extends_class_shapes() {
    let result = parse(
        4,
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

abstract interface class ApiContract {}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.library.classes.len(), 3);
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
    assert_eq!(result.library.classes[2].name, "ApiContract");
    assert!(result.library.classes[2].is_abstract);
    assert!(result.library.classes[2].is_interface);
}

#[test]
fn extracts_classes_with_mixin_clauses_and_ignores_plain_mixin_declarations() {
    let result = parse(
        5,
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
    let result = parse(
        6,
        r#"
part 'mixin_target.g.dart';

@Derive([ToString()])
abstract mixin class MixinTarget {
  const MixinTarget(this.id);

  final String id;
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.library.classes.len(), 1);
    assert_eq!(result.library.classes[0].kind, ParsedClassKind::MixinClass);
    assert!(result.library.classes[0].is_abstract);
}
