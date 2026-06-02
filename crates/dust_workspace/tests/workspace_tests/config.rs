use dust_workspace::{PackageConfigKind, load_dust_config, load_package_config};
use tempfile::tempdir;

use crate::support::write_file;

#[test]
fn load_package_config_requires_real_file() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");

    let error = load_package_config(root.path()).unwrap_err();
    assert!(error.message.contains("missing package configuration"));

    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");
    let config = load_package_config(root.path()).unwrap();

    assert_eq!(
        config.path,
        root.path().join(".dart_tool/package_config.json")
    );
    assert_eq!(config.kind, PackageConfigKind::Direct);
}

#[test]
fn load_package_config_uses_shared_workspace_config_for_member_package() {
    let root = tempdir().unwrap();
    let workspace_root = root.path();
    let package_root = workspace_root.join("examples/product_showcase");
    write_file(
        &workspace_root.join("pubspec.yaml"),
        "name: dust_workspace\n",
    );
    write_file(
        &workspace_root.join(".dart_tool/package_config.json"),
        "{\"configVersion\":2}\n",
    );
    write_file(
        &package_root.join("pubspec.yaml"),
        "name: product_showcase\n",
    );
    write_file(
        &package_root.join(".dart_tool/package_graph.json"),
        "{\"configVersion\":1}\n",
    );

    let config = load_package_config(&package_root).unwrap();

    assert_eq!(
        config.path,
        workspace_root.join(".dart_tool/package_config.json")
    );
    assert_eq!(
        config.kind,
        PackageConfigKind::WorkspaceShared {
            package_graph_path: package_root.join(".dart_tool/package_graph.json"),
        }
    );
}

#[test]
fn load_package_config_uses_workspace_ref_for_newer_pub_workspaces() {
    let root = tempdir().unwrap();
    let workspace_root = root.path();
    let package_root = workspace_root.join("examples/dust_db_example");
    write_file(
        &workspace_root.join("pubspec.yaml"),
        "name: dust_workspace\n",
    );
    write_file(
        &workspace_root.join(".dart_tool/package_config.json"),
        "{\"configVersion\":2}\n",
    );
    write_file(
        &package_root.join("pubspec.yaml"),
        "name: dust_db_example\n",
    );
    write_file(
        &package_root.join(".dart_tool/pub/workspace_ref.json"),
        "{\"workspaceRoot\":\"../..\"}\n",
    );

    let config = load_package_config(&package_root).unwrap();

    assert_eq!(
        config.path,
        workspace_root.join(".dart_tool/package_config.json")
    );
    assert_eq!(
        config.kind,
        PackageConfigKind::WorkspaceShared {
            package_graph_path: package_root.join(".dart_tool/pub/workspace_ref.json"),
        }
    );
}

#[test]
fn load_package_config_reports_missing_shared_workspace_config() {
    let root = tempdir().unwrap();
    let package_root = root.path().join("examples/product_showcase");
    write_file(
        &package_root.join("pubspec.yaml"),
        "name: product_showcase\n",
    );
    write_file(
        &package_root.join(".dart_tool/package_graph.json"),
        "{\"configVersion\":1}\n",
    );

    let error = load_package_config(&package_root).unwrap_err();

    assert!(error.message.contains("workspace member"));
    assert!(
        error
            .message
            .contains(package_root.to_string_lossy().as_ref())
    );
    assert!(error.message.contains(".dart_tool/package_config.json"));
}

#[test]
fn load_dust_config_rejects_invalid_suffixes() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(
        &root.path().join("dust.yaml"),
        "outputs:\n  primary_suffix: generated.dart\n",
    );

    let error = load_dust_config(root.path()).unwrap_err();
    assert!(error.message.contains("outputs.primary_suffix"));
}
