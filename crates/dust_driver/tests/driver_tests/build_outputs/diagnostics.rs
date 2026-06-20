use dust_diagnostics::render_to_string_with_files;
use dust_driver::{BuildRequest, run_build};

use crate::support::{make_workspace, write_file};

#[test]
fn build_rejects_invalid_serde_using_values() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/audit.dart"),
        "part 'audit.g.dart';\n\
         @Derive([Serialize(), Deserialize()])\n\
         class Audit {\n\
           const Audit({required this.createdAt});\n\
           @SerDe(using: DateTimeCodec)\n\
           final DateTime createdAt;\n\
           factory Audit.fromJson(Map<String, Object?> json) => _$AuditFromJson(json);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    assert!(result.diagnostic_files.is_empty());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains(
            "field `createdAt` uses suspicious `SerDe(using: ...)` type reference `DateTimeCodec`",
        )
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.notes.iter().any(|note| {
            note.contains("Use a codec object such as `const UnixEpochDateTimeCodec()`")
        })
    }));
}

#[test]
fn build_keeps_source_context_for_labeled_diagnostics() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @Derive([ToString(), UnknownTrait()])\n\
         class User {\n\
           final String id;\n\
           const User(this.id);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert_eq!(result.diagnostic_files.len(), 1);
    let file = &result.diagnostic_files[0];
    assert_eq!(file.path, workspace.path().join("lib/user.dart"));
    assert_eq!(file.file_id, result.diagnostics[0].labels[0].file_id);
    assert!(
        file.source_text()
            .contains("@Derive([ToString(), UnknownTrait()])")
    );

    let rendered = render_to_string_with_files(&result.diagnostics[0], &[file.render_context()]);
    assert!(rendered.contains(&format!("  --> {}:3:1", file.path.display())));
    assert!(rendered.contains("3 | @Derive([ToString(), UnknownTrait()])"));
    assert!(rendered.contains("annotation member is not owned by any registered symbol"));
}
