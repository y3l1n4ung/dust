use std::fs;

use dust_workspace::{
    PackageConfigKind, detect_workspace_root, discover_libraries, discover_workspace,
    load_dust_config, load_package_config,
};
use tempfile::tempdir;

fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

const TEST_ANNOTATIONS: &[&str] = &["Derive", "ToString", "Client"];

#[test]
fn detects_workspace_root_from_nested_directory_and_file_path() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");
    let nested_dir = root.path().join("lib/src/models");
    fs::create_dir_all(&nested_dir).unwrap();
    let nested_file = nested_dir.join("user.dart");
    write_file(&nested_file, "class User {}\n");

    let from_dir = detect_workspace_root(&nested_dir).unwrap();
    let from_file = detect_workspace_root(&nested_file).unwrap();

    assert_eq!(from_dir, root.path());
    assert_eq!(from_file, root.path());
}

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
fn discover_libraries_scans_recursively_in_stable_order() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/src/team.dart"),
        "part 'team.g.dart';\n@Derive([ToString()])\nclass Team {}\n",
    );
    write_file(
        &root.path().join("lib/user.dart"),
        "part 'user.g.dart';\n@ToString()\nclass User {}\n",
    );
    write_file(&root.path().join("lib/user.g.dart"), "// generated\n");
    write_file(
        &root.path().join("lib/ignored.dart"),
        "part 'ignored.g.dart';\nclass Ignored {}\n",
    );

    let libraries = discover_libraries(root.path(), TEST_ANNOTATIONS).unwrap();

    assert_eq!(libraries.len(), 2);
    assert_eq!(
        libraries[0].source_path,
        root.path().join("lib/src/team.dart")
    );
    assert_eq!(
        libraries[0].output_path,
        root.path().join("lib/src/team.g.dart")
    );
    assert_eq!(libraries[1].source_path, root.path().join("lib/user.dart"));
    assert_eq!(
        libraries[1].output_path,
        root.path().join("lib/user.g.dart")
    );
}

#[test]
fn discover_libraries_uses_configured_primary_suffix() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");
    write_file(
        &root.path().join("dust.yaml"),
        "outputs:\n  primary_suffix: .d.dart\n",
    );
    write_file(
        &root.path().join("lib/client.dart"),
        "part 'client.d.dart';\n@Client()\nabstract class ApiClient {}\n",
    );
    write_file(&root.path().join("lib/client.d.dart"), "// generated\n");

    let libraries = discover_libraries(root.path(), TEST_ANNOTATIONS).unwrap();

    assert_eq!(libraries.len(), 1);
    assert_eq!(
        libraries[0].output_path,
        root.path().join("lib/client.d.dart")
    );
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

#[test]
fn discover_libraries_accepts_double_quoted_part_and_direct_annotations() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/client.dart"),
        "part \"client.g.dart\";\n@Client()\nabstract class ApiClient {}\n",
    );

    let libraries = discover_libraries(root.path(), TEST_ANNOTATIONS).unwrap();

    assert_eq!(libraries.len(), 1);
    assert_eq!(
        libraries[0].source_path,
        root.path().join("lib/client.dart")
    );
}

#[test]
fn discover_libraries_ignores_override_and_unknown_annotations() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/view.dart"),
        "part 'view.g.dart';\nclass Demo {\n  @override\n  String toString() => 'demo';\n}\n",
    );
    write_file(
        &root.path().join("lib/view_model.dart"),
        "part 'view_model.g.dart';\n@ViewModel()\nclass DemoViewModel {}\n",
    );

    let libraries = discover_libraries(root.path(), TEST_ANNOTATIONS).unwrap();

    assert!(libraries.is_empty());
}

#[test]
fn discover_workspace_composes_root_config_and_library_scan() {
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
        &workspace_root.join("lib/root.dart"),
        "part 'root.g.dart';\n@ToString()\nclass Root {}\n",
    );
    write_file(
        &package_root.join("pubspec.yaml"),
        "name: product_showcase\n",
    );
    write_file(
        &package_root.join(".dart_tool/package_graph.json"),
        "{\"configVersion\":1}\n",
    );
    write_file(
        &package_root.join("lib/models/user.dart"),
        "part 'user.g.dart';\n@Derive([ToString(), Eq()])\nclass User {}\n",
    );

    let nested = package_root.join("lib/models");
    let plan = discover_workspace(&nested, TEST_ANNOTATIONS).unwrap();

    assert_eq!(plan.package_root, package_root);
    assert_eq!(plan.cache_root, plan.package_root);
    assert_eq!(plan.package_name, "product_showcase");
    assert_eq!(
        plan.package_config.path,
        workspace_root.join(".dart_tool/package_config.json")
    );
    assert_eq!(
        plan.package_config.kind,
        PackageConfigKind::WorkspaceShared {
            package_graph_path: plan.package_root.join(".dart_tool/package_graph.json"),
        }
    );
    assert_eq!(plan.libraries.len(), 1);
    assert_eq!(
        plan.libraries[0].output_path,
        plan.package_root.join("lib/models/user.g.dart")
    );
    assert_eq!(plan.dust_config.outputs.primary_suffix, ".g.dart");
}
