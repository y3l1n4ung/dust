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

#[test]
fn build_uses_workspace_serde_json_capability_facts() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/profile.dart"),
        r#"part 'profile.g.dart';

@Derive([Serialize(), Deserialize()])
class JsonProfile with _$JsonProfile {
  const JsonProfile({required this.id});

  factory JsonProfile.fromJson(Map<String, Object?> json) =>
      _$JsonProfileFromJson(json);

  final String id;
}
"#,
    );
    write_file(
        &workspace.path().join("lib/account.dart"),
        r#"import 'profile.dart';

part 'account.g.dart';

@Derive([Serialize(), Deserialize()])
class JsonAccount with _$JsonAccount {
  const JsonAccount({required this.profile, required this.bad});

  factory JsonAccount.fromJson(Map<String, Object?> json) =>
      _$JsonAccountFromJson(json);

  final JsonProfile profile;
  final JsonBad bad;
}

class JsonBad {
  const JsonBad();
}
"#,
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    let messages = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec![
            "`Serialize` requires `JsonBad.toJson()` or deriving `Serialize`/using `SerDe(codec: ...)` for `JsonAccount.bad`",
            "`Deserialize` requires `JsonBad.fromJson(Map<String, Object?>)` or deriving `Deserialize`/using `SerDe(codec: ...)` for `JsonAccount.bad`",
        ]
    );
}

#[test]
fn build_accepts_workspace_handwritten_json_members() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/profile.dart"),
        r#"part 'profile.g.dart';

@Derive([Serialize()])
enum JsonMarker { one }

class HandwrittenProfile {
  const HandwrittenProfile(this.id);

  factory HandwrittenProfile.fromJson(Map<String, Object?> json) {
    return HandwrittenProfile(json['id'] as String);
  }

  final String id;

  Map<String, Object?> toJson() => {'id': id};
}
"#,
    );
    write_file(
        &workspace.path().join("lib/account.dart"),
        r#"import 'profile.dart';

part 'account.g.dart';

@Derive([Serialize(), Deserialize()])
class JsonAccount with _$JsonAccount {
  const JsonAccount({required this.profile});

  factory JsonAccount.fromJson(Map<String, Object?> json) =>
      _$JsonAccountFromJson(json);

  final HandwrittenProfile profile;
}
"#,
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert_eq!(result.diagnostics, Vec::new());
}

#[test]
fn build_reports_missing_workspace_json_direction_only() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/profile.dart"),
        r#"part 'profile.g.dart';

@Derive([Serialize(), Deserialize()])
enum JsonStatus { active }

class JsonSerializeOnly {
  const JsonSerializeOnly(this.id);

  final String id;

  Map<String, Object?> toJson() => {'id': id};
}
"#,
    );
    write_file(
        &workspace.path().join("lib/account.dart"),
        r#"import 'profile.dart';

part 'account.g.dart';

@Derive([Serialize(), Deserialize()])
class JsonAccount with _$JsonAccount {
  const JsonAccount({required this.status, required this.profile});

  factory JsonAccount.fromJson(Map<String, Object?> json) =>
      _$JsonAccountFromJson(json);

  final JsonStatus status;
  final JsonSerializeOnly profile;
}
"#,
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    let messages = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "`Deserialize` requires `JsonSerializeOnly.fromJson(Map<String, Object?>)` or deriving `Deserialize`/using `SerDe(codec: ...)` for `JsonAccount.profile`",
        ]
    );
}
