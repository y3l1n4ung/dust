use dust_driver::{BuildRequest, CheckRequest, run_build, run_check};

use super::support::{
    DustImport, make_workspace, write_dust_file, write_file, write_resolved_dust_packages,
};

#[test]
fn build_allows_compatible_dust_package_versions() {
    let workspace = make_workspace();
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.1.3")]);
    write_dust_file(
        &workspace.path().join("lib/user.dart"),
        &[DustImport::Derive],
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

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(workspace.path().join("lib/user.g.dart").exists());
}

#[test]
fn build_rejects_too_old_dust_package_before_writing_outputs() {
    let workspace = make_workspace();
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.1.2")]);
    write_dust_file(
        &workspace.path().join("lib/user.dart"),
        &[DustImport::Derive],
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
    let diagnostic = &result.diagnostics[0];
    assert!(
        diagnostic
            .message
            .contains("unsupported Dust package version")
    );
    assert!(diagnostic.message.contains("CLI 0.1.4"));
    assert!(diagnostic.message.contains("`dust_dart` >=0.1.3 <0.2.0"));
    assert!(diagnostic.message.contains("resolves 0.1.2"));
    assert!(
        diagnostic
            .notes
            .contains(&"Upgrade the Dust package dependency in pubspec.yaml.".to_owned())
    );
    assert!(!workspace.path().join("lib/user.g.dart").exists());
}

#[test]
fn check_rejects_too_new_dust_package_before_processing_libraries() {
    let workspace = make_workspace();
    write_resolved_dust_packages(workspace.path(), &[("dust_flutter", "0.2.0")]);
    write_dust_file(
        &workspace.path().join("lib/counter.dart"),
        &[DustImport::State],
        "part 'counter.g.dart';\n\
         @ViewModel()\n\
         class CounterViewModel {}\n",
    );

    let result = run_check(CheckRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    let diagnostic = &result.diagnostics[0];
    assert!(
        diagnostic
            .message
            .contains("unsupported Dust package version")
    );
    assert!(diagnostic.message.contains("`dust_flutter` >=0.1.4 <0.2.0"));
    assert!(diagnostic.message.contains("resolves 0.2.0"));
    assert!(diagnostic.notes.contains(
        &"Upgrade the Dust CLI first, or pin the package to a supported range.".to_owned()
    ));
    assert!(result.checked_libraries.is_empty());
}

#[test]
fn build_ignores_resolved_dust_package_that_source_does_not_use() {
    let workspace = make_workspace();
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.1.2")]);
    write_file(
        &workspace.path().join("lib/user.dart"),
        "class User {\n\
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
    assert!(result.build_artifacts.is_empty());
}

#[test]
fn build_requires_visible_dust_import_for_annotation_discovery() {
    let workspace = make_workspace();
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.1.3")]);
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

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(result.build_artifacts.is_empty());
    assert!(!workspace.path().join("lib/user.g.dart").exists());
}
