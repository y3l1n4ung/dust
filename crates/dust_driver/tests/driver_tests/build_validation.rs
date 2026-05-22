use std::fs;

use dust_driver::{BuildRequest, run_build};

use super::support::{generated_output, make_workspace, write_file};

#[test]
fn build_supports_abstract_and_mixin_clause_shapes_without_unrelated_warnings() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/entity.dart"),
        "part 'entity.g.dart';\n\
         mixin AuditStamp {\n\
           String auditLabel() => 'audited';\n\
         }\n\
         class CatalogNode {\n\
           const CatalogNode();\n\
         }\n\
         @Derive([ToString(), Eq()])\n\
         abstract class Entity extends CatalogNode with AuditStamp {\n\
           final String id;\n\
           const Entity(this.id);\n\
         }\n\
         class EntityView extends Entity {\n\
           const EntityView(super.id);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/tagged_value.dart"),
        "part 'tagged_value.g.dart';\n\
         mixin LabelStamp {\n\
           String labelKind() => 'tagged';\n\
         }\n\
         @Derive([ToString(), Eq(), CopyWith()])\n\
         class TaggedValue with LabelStamp {\n\
           final String code;\n\
           final List<String> aliases;\n\
           const TaggedValue({required this.code, required this.aliases});\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    let entity_output = fs::read_to_string(workspace.path().join("lib/entity.g.dart")).unwrap();
    let tagged_output =
        fs::read_to_string(workspace.path().join("lib/tagged_value.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(
        result.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("could not infer constructor parameter type")),
        "{:?}",
        result.diagnostics
    );
    assert_eq!(
        entity_output,
        generated_output(
            r#"part of 'entity.dart';

mixin _$Entity {
  @override
  String toString() {
    final self = this as Entity;
    return 'Entity('
        'id: ${self.id}'
        ')';
  }

  @override
  bool operator ==(Object other) {
    final self = this as Entity;
    return identical(this, other) ||
        other is Entity &&
            runtimeType == other.runtimeType &&
            other.id == self.id;
  }

  @override
  int get hashCode {
    final self = this as Entity;
    return Object.hashAll([
      runtimeType,
      self.id,
    ]);
  }
}
"#
        )
    );
    assert_eq!(
        tagged_output,
        generated_output(
            r#"part of 'tagged_value.dart';

const DeepCollectionEquality _deepCollectionEquality = DeepCollectionEquality();

mixin _$TaggedValue {
  @override
  String toString() {
    final self = this as TaggedValue;
    return 'TaggedValue('
        'code: ${self.code}, '
        'aliases: ${self.aliases}'
        ')';
  }

  @override
  bool operator ==(Object other) {
    final self = this as TaggedValue;
    return identical(this, other) ||
        other is TaggedValue &&
            runtimeType == other.runtimeType &&
            other.code == self.code &&
            _deepCollectionEquality.equals(other.aliases, self.aliases);
  }

  @override
  int get hashCode {
    final self = this as TaggedValue;
    return Object.hashAll([
      runtimeType,
      self.code,
      _deepCollectionEquality.hash(self.aliases),
    ]);
  }

  TaggedValue copyWith({
    String? code,
    List<String>? aliases,
  }) {
    final self = this as TaggedValue;
    final nextAliases = List<String>.of(aliases ?? self.aliases);

    return TaggedValue(
      code: code ?? self.code,
      aliases: nextAliases,
    );
  }
}
"#
        )
    );
}

#[test]
fn build_includes_inherited_fields_for_annotated_subclasses() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/entity.dart"),
        "part 'entity.g.dart';\n\
         @Derive([ToString(), Eq()])\n\
         abstract class Entity with _$Entity {\n\
           final String id;\n\
           const Entity(this.id);\n\
         }\n\
         @Derive([ToString(), Eq(), CopyWith()])\n\
         class DetailedEntity extends Entity with _$DetailedEntity {\n\
           final String label;\n\
           final List<String> tags;\n\
           const DetailedEntity(super.id, {required this.label, required this.tags});\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    let output = fs::read_to_string(workspace.path().join("lib/entity.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(
        result.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("could not infer constructor parameter type")),
        "{:?}",
        result.diagnostics
    );
    assert_eq!(
        output,
        generated_output(
            r#"part of 'entity.dart';

const DeepCollectionEquality _deepCollectionEquality = DeepCollectionEquality();

mixin _$Entity {
  @override
  String toString() {
    final self = this as Entity;
    return 'Entity('
        'id: ${self.id}'
        ')';
  }

  @override
  bool operator ==(Object other) {
    final self = this as Entity;
    return identical(this, other) ||
        other is Entity &&
            runtimeType == other.runtimeType &&
            other.id == self.id;
  }

  @override
  int get hashCode {
    final self = this as Entity;
    return Object.hashAll([
      runtimeType,
      self.id,
    ]);
  }
}

mixin _$DetailedEntity {
  @override
  String toString() {
    final self = this as DetailedEntity;
    return 'DetailedEntity('
        'id: ${self.id}, '
        'label: ${self.label}, '
        'tags: ${self.tags}'
        ')';
  }

  @override
  bool operator ==(Object other) {
    final self = this as DetailedEntity;
    return identical(this, other) ||
        other is DetailedEntity &&
            runtimeType == other.runtimeType &&
            other.id == self.id &&
            other.label == self.label &&
            _deepCollectionEquality.equals(other.tags, self.tags);
  }

  @override
  int get hashCode {
    final self = this as DetailedEntity;
    return Object.hashAll([
      runtimeType,
      self.id,
      self.label,
      _deepCollectionEquality.hash(self.tags),
    ]);
  }

  DetailedEntity copyWith({
    String? id,
    String? label,
    List<String>? tags,
  }) {
    final self = this as DetailedEntity;
    final nextTags = List<String>.of(tags ?? self.tags);

    return DetailedEntity(
      id ?? self.id,
      label: label ?? self.label,
      tags: nextTags,
    );
  }
}
"#
        )
    );
}

#[test]
fn build_rejects_mixin_class_targets_with_clear_diagnostic() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/mixin_target.dart"),
        "part 'mixin_target.g.dart';\n\
         @Derive([ToString(), CopyWith()])\n\
         mixin class MixinTarget {\n\
           final String id;\n\
           const MixinTarget(this.id);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
    });

    assert!(result.has_errors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("does not support `mixin class` targets like `MixinTarget`")
    }));
    assert!(!workspace.path().join("lib/mixin_target.g.dart").exists());
}
