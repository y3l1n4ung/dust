use std::fs;

use dust_workspace::{
    detect_workspace_root, discover_libraries, discover_workspace, load_package_config,
};
use tempfile::tempdir;

fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

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
}

#[test]
fn discover_libraries_scans_recursively_in_stable_order() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/src/team.dart"),
        "part 'team.g.dart';\n@Derive([Debug()])\nclass Team {}\n",
    );
    write_file(
        &root.path().join("lib/user.dart"),
        "part 'user.g.dart';\n@Debug()\nclass User {}\n",
    );
    write_file(&root.path().join("lib/user.g.dart"), "// generated\n");
    write_file(
        &root.path().join("lib/ignored.dart"),
        "part 'ignored.g.dart';\nclass Ignored {}\n",
    );

    let libraries = discover_libraries(root.path()).unwrap();

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
fn discover_libraries_accepts_double_quoted_part_and_direct_annotations() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");

    write_file(
        &root.path().join("lib/client.dart"),
        "part \"client.g.dart\";\n@Client()\nabstract class ApiClient {}\n",
    );

    let libraries = discover_libraries(root.path()).unwrap();

    assert_eq!(libraries.len(), 1);
    assert_eq!(
        libraries[0].source_path,
        root.path().join("lib/client.dart")
    );
}

#[test]
fn discover_workspace_composes_root_config_and_library_scan() {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");
    write_file(
        &root.path().join("lib/models/user.dart"),
        "part 'user.g.dart';\n@Derive([Debug(), Eq()])\nclass User {}\n",
    );

    let nested = root.path().join("lib/models");
    let plan = discover_workspace(&nested).unwrap();

    assert_eq!(plan.root, root.path());
    assert_eq!(
        plan.package_config.path,
        root.path().join(".dart_tool/package_config.json")
    );
    assert_eq!(plan.libraries.len(), 1);
    assert_eq!(
        plan.libraries[0].output_path,
        root.path().join("lib/models/user.g.dart")
    );
}
