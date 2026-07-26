use std::fs;

use dust_plugin_api::GENERATED_HEADER;
use tempfile::tempdir;

pub(crate) enum DustImport {
    Derive,
    Http,
    Route,
    State,
}

pub(crate) fn generated_output(body: &str) -> String {
    format!("{GENERATED_HEADER}\n{body}")
}

pub(crate) fn write_dust_file(path: &std::path::Path, imports: &[DustImport], contents: &str) {
    let import_block = imports
        .iter()
        .map(|import| match import {
            DustImport::Derive => "import 'package:dust_dart/derive.dart';\n",
            DustImport::Http => "import 'package:dust_dart/http.dart';\n",
            DustImport::Route => "import 'package:dust_flutter/route.dart';\n",
            DustImport::State => "import 'package:dust_flutter/state.dart';\n",
        })
        .collect::<String>();
    write_file(path, &format!("{import_block}{contents}"));
}

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

pub(crate) fn write_resolved_dust_packages(root: &std::path::Path, packages: &[(&str, &str)]) {
    for (name, version) in packages {
        write_file(
            &root.join(format!("deps/{name}/pubspec.yaml")),
            &format!("name: {name}\nversion: {version}\n"),
        );
    }

    let package_entries = packages
        .iter()
        .map(|(name, _)| {
            format!(
                r#"{{"name":"{name}","rootUri":"../deps/{name}","packageUri":"lib/","languageVersion":"3.6"}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let suffix = if package_entries.is_empty() {
        String::new()
    } else {
        format!(",{package_entries}")
    };
    write_file(
        &root.join(".dart_tool/package_config.json"),
        &format!(
            r#"{{"configVersion":2,"packages":[{{"name":"dust_test","rootUri":"../","packageUri":"lib/","languageVersion":"3.6"}}{suffix}]}}"#
        ),
    );
}
