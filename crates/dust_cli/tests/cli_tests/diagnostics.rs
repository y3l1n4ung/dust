use dust_cli::run_cli;

use super::helpers::{make_workspace, write_file};

#[test]
fn cli_build_renders_warning_details() {
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

    let run = run_cli(["build", "--root", workspace.path().to_str().unwrap()]);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert!(run.stderr.is_empty());
    assert!(
        run.stdout
            .contains("diagnostics  errors: 0  warnings: 1  notes: 0")
    );
    assert!(
        run.stdout
            .contains("warning: unknown derive trait or config `UnknownTrait`")
    );
    assert!(run.stdout.contains(&format!(
        "{}:3:",
        workspace.path().join("lib/user.dart").display()
    )));
    assert!(
        run.stdout
            .contains("3 | @Derive([ToString(), UnknownTrait()])")
    );
    assert!(
        run.stdout
            .contains("annotation member is not owned by any registered symbol")
    );
    assert!(run.stdout.contains("^^^^"));
}

#[test]
fn cli_build_renders_error_notes() {
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

    let run = run_cli(["build", "--root", workspace.path().to_str().unwrap()]);

    assert_eq!(run.exit_code, 1);
    assert!(run.stdout.is_empty());
    assert!(
        run.stderr
            .contains("diagnostics  errors: 1  warnings: 0  notes: 0")
    );
    assert!(run.stderr.contains(
        "error: field `createdAt` uses suspicious `SerDe(using: ...)` type reference `DateTimeCodec`"
    ));
    assert!(run.stderr.contains(
        "Use a codec object such as `const UnixEpochDateTimeCodec()` or `unixEpochDateTimeCodec`."
    ));
}
