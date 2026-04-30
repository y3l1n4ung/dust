use std::fs;

use dust_driver::{BuildRequest, run_build};

use super::support::{make_workspace, write_file};

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
    assert!(entity_output.contains("mixin _$EntityDust {"));
    assert!(entity_output.contains("Entity get _dustSelf => this as Entity;"));
    assert!(entity_output.contains("other is Entity"));
    assert!(tagged_output.contains("mixin _$TaggedValueDust {"));
    assert!(tagged_output.contains("_dustDeepCollectionEquality.equals"));
    assert!(tagged_output.contains("TaggedValue copyWith({"));
    assert!(
        tagged_output
            .contains("final nextAliases = List<String>.of(aliases ?? _dustSelf.aliases);")
    );
}

#[test]
fn build_includes_inherited_fields_for_annotated_subclasses() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/entity.dart"),
        "part 'entity.g.dart';\n\
         @Derive([ToString(), Eq()])\n\
         abstract class Entity with _$EntityDust {\n\
           final String id;\n\
           const Entity(this.id);\n\
         }\n\
         @Derive([ToString(), Eq(), CopyWith()])\n\
         class DetailedEntity extends Entity with _$DetailedEntityDust {\n\
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
    assert!(output.contains("mixin _$DetailedEntityDust {"));
    assert!(output.contains("DetailedEntity get _dustSelf => this as DetailedEntity;"));
    assert!(output.contains("return 'DetailedEntity('"));
    assert!(output.contains("'id: ${_dustSelf.id}, '"));
    assert!(output.contains("'label: ${_dustSelf.label}, '"));
    assert!(output.contains("'tags: ${_dustSelf.tags}'"));
    assert!(output.contains("other.id == _dustSelf.id"));
    assert!(output.contains("DetailedEntity copyWith({"));
    assert!(output.contains("final nextTags = List<String>.of(tags ?? _dustSelf.tags);"));
    assert!(output.contains("return DetailedEntity("));
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
