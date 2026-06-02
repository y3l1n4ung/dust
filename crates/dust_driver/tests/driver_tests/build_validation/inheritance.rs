use std::fs;

use dust_driver::{BuildRequest, run_build};

use crate::support::{generated_output, make_workspace, write_file};

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
        db: Default::default(),
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
