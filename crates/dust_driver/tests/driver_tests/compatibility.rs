use dust_driver::{BuildRequest, CheckRequest, run_build, run_check};

use super::support::{
    DustImport, make_workspace, write_dust_file, write_file, write_resolved_dust_packages,
};

#[test]
fn build_allows_compatible_dust_package_versions() {
    let workspace = make_workspace();
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.2.0")]);
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
    assert!(diagnostic.message.contains("CLI 0.2.0"));
    assert!(diagnostic.message.contains("`dust_dart` >=0.2.0 <0.3.0"));
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
    write_resolved_dust_packages(workspace.path(), &[("dust_flutter", "0.3.0")]);
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
    assert!(diagnostic.message.contains("`dust_flutter` >=0.2.0 <0.3.0"));
    assert!(diagnostic.message.contains("resolves 0.3.0"));
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
    write_resolved_dust_packages(workspace.path(), &[("dust_dart", "0.2.0")]);
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

#[test]
fn build_rejects_too_old_dust_db_postgres() {
    let workspace = make_workspace();
    write_resolved_dust_packages(
        workspace.path(),
        &[("dust_dart", "0.2.0"), ("dust_db_postgres", "0.1.9")],
    );
    write_dust_file(
        &workspace.path().join("lib/orders.dart"),
        &[DustImport::Derive, DustImport::DbPostgres],
        "part 'orders.g.dart';\n\
         @ToString()\n\
         class Order {\n\
           final String id;\n\
           const Order(this.id);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors(), "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert!(
        diagnostic.message.contains("`dust_db_postgres`"),
        "{diagnostic:?}"
    );
    assert!(
        diagnostic.message.contains("resolves 0.1.9"),
        "{diagnostic:?}"
    );
    assert!(!workspace.path().join("lib/orders.g.dart").exists());
}

#[test]
fn every_database_runtime_is_known_to_discovery_and_the_contract() {
    // Three lists have to agree for a database runtime to be version-checked:
    // the DB plugin's dialect registry, the import table workspace discovery
    // scans, and the CLI's compatibility contract. When `dust_db_postgres` was
    // added to the last two but not the first, `dust doctor` reported it as
    // unused and an incompatible version was accepted silently.
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../compatibility/dust-cli-packages.json"
    ))
    .expect("the compatibility contract parses");
    let constraints = contract["entries"][0]["packageConstraints"]
        .as_object()
        .expect("the newest CLI row lists package constraints");

    for package in dust_db_plugin::database_runtime_packages() {
        assert!(
            dust_workspace::dust_runtime_packages().contains(&package),
            "`{package}` is a database runtime but workspace discovery does not scan for it, \
             so nothing records that a project uses it"
        );
        assert!(
            constraints.contains_key(package),
            "`{package}` is a database runtime with no compatibility rule for this CLI"
        );
    }
}
