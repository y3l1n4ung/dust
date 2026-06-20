use std::fs;

use dust_plugin_api::GENERATED_HEADER;
use tempfile::tempdir;

pub(crate) fn generated_output(body: &str) -> String {
    format!("{GENERATED_HEADER}\n{body}")
}

pub(crate) fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, fixture_contents(path, contents)).expect("write file");
}

fn fixture_contents(path: &std::path::Path, contents: &str) -> String {
    if path.extension().and_then(|ext| ext.to_str()) != Some("dart")
        || contents.contains("package:dust_dart/")
        || contents.contains("package:dust_flutter/")
    {
        return contents.to_owned();
    }

    let Some(import) = fixture_dust_import(contents) else {
        return contents.to_owned();
    };

    format!("{import}{contents}")
}

fn fixture_dust_import(contents: &str) -> Option<&'static str> {
    if contents.contains("@Route") || contents.contains("@Router") {
        Some("import 'package:dust_flutter/route.dart';\n")
    } else if contents.contains("@ViewModel") {
        Some("import 'package:dust_flutter/state.dart';\n")
    } else if contents.contains("@HttpClient")
        || contents.contains("@GenerateTest")
        || contents.contains("@GET")
        || contents.contains("@POST")
    {
        Some("import 'package:dust_dart/http.dart';\n")
    } else if contents.contains("@Sqlx")
        || contents.contains("@SqlxDatabase")
        || contents.contains("@SqlxDao")
        || contents.contains("FromRow()")
    {
        Some("import 'package:dust_dart/db.dart';\n")
    } else if contents.contains("@Derive")
        || contents.contains("@ToString")
        || contents.contains("@Debug")
        || contents.contains("@Eq")
        || contents.contains("@CopyWith")
        || contents.contains("@Validate")
        || contents.contains("@SerDe")
        || contents.contains("Serialize()")
        || contents.contains("Deserialize()")
    {
        Some("import 'package:dust_dart/derive.dart';\n")
    } else {
        None
    }
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
