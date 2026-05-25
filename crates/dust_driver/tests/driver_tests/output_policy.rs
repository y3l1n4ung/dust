use dust_driver::{BuildRequest, run_build};

use super::support::{make_workspace, write_file};

#[test]
fn build_uses_configured_primary_suffix() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "outputs:\n  primary_suffix: .d.dart\n",
    );
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.d.dart';\n\
         @ToString()\n\
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

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(workspace.path().join("lib/user.d.dart").exists());
    assert!(!workspace.path().join("lib/user.g.dart").exists());
}

#[test]
fn build_reports_part_suffix_mismatches_from_dust_config() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "outputs:\n  primary_suffix: .d.dart\n",
    );
    write_file(
        &workspace.path().join("lib/user.dart"),
        "part 'user.g.dart';\n\
         @ToString()\n\
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

    assert!(result.has_errors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("does not match expected `user.d.dart`")
    }));
    assert!(!workspace.path().join("lib/user.d.dart").exists());
}
