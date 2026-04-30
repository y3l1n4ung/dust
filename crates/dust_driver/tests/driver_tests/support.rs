use std::fs;

use tempfile::tempdir;

pub(crate) fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

pub(crate) fn make_workspace() -> tempfile::TempDir {
    let root = tempdir().unwrap();
    write_file(&root.path().join("pubspec.yaml"), "name: dust_test\n");
    write_file(&root.path().join(".dart_tool/package_config.json"), "{}\n");
    root
}

pub(crate) fn make_pub_workspace_member() -> (tempfile::TempDir, std::path::PathBuf) {
    let root = tempdir().unwrap();
    write_file(
        &root.path().join("pubspec.yaml"),
        "name: dust_workspace\nworkspace:\n  - examples/product_showcase\n",
    );
    write_file(
        &root.path().join(".dart_tool/package_config.json"),
        "{\"configVersion\":2,\"packages\":[]}\n",
    );
    let package_root = root.path().join("examples/product_showcase");
    write_file(
        &package_root.join("pubspec.yaml"),
        "name: product_showcase\nresolution: workspace\n",
    );
    write_file(
        &package_root.join(".dart_tool/package_graph.json"),
        "{\"configVersion\":1,\"roots\":[\"product_showcase\"],\"packages\":[]}\n",
    );
    (root, package_root)
}
