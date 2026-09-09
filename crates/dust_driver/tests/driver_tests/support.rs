use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use dust_plugin_api::GENERATED_HEADER;
use tempfile::tempdir;

pub(crate) enum DustImport {
    Derive,
    Http,
    Route,
    State,
    DbPostgres,
}

pub(crate) fn generated_output(body: &str) -> String {
    format!("{GENERATED_HEADER}\n{body}")
}

/// Renders a Dust library source with the requested import block.
fn dust_source(imports: &[DustImport], contents: &str) -> String {
    let import_block = imports
        .iter()
        .map(|import| match import {
            DustImport::Derive => "import 'package:dust_dart/derive.dart';\n",
            DustImport::Http => "import 'package:dust_dart/http.dart';\n",
            DustImport::Route => "import 'package:dust_flutter/route.dart';\n",
            DustImport::State => "import 'package:dust_flutter/state.dart';\n",
            DustImport::DbPostgres => "import 'package:dust_db_postgres/dust_db_postgres.dart';\n",
        })
        .collect::<String>();
    format!("{import_block}{contents}")
}

pub(crate) fn write_dust_file(path: &std::path::Path, imports: &[DustImport], contents: &str) {
    write_file(path, &dust_source(imports, contents));
}

/// Replaces a Dust library in one step, for a writer racing a watch rescan.
pub(crate) fn replace_dust_file_atomically(path: &Path, imports: &[DustImport], contents: &str) {
    replace_file_atomically(path, &dust_source(imports, contents));
}

pub(crate) fn write_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write file");
}

/// Blocks until [path] exists, so a concurrent writer can order itself against
/// work the driver has finished rather than against the clock.
///
/// A watch run takes its baseline snapshot before the initial build writes any
/// output, so a generated file appearing means any later write is a change the
/// watcher can still see. Sleeping a fixed interval instead is a race: under
/// load the initial build outruns the sleep, the write lands before the
/// baseline, and no rebuild is ever observed.
pub(crate) fn wait_for_path(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for `{}`",
            path.display()
        );
        thread::sleep(Duration::from_millis(2));
    }
}

/// Replaces [path] in one step, so a concurrent reader never sees half a file.
///
/// The watch loop rescans while these tests write, and a partially written
/// source or package config is read as a parse failure rather than as the
/// change the test is making.
pub(crate) fn replace_file_atomically(path: &Path, contents: &str) {
    let staging = path.with_file_name(format!(
        "{}.dust-test-next",
        path.file_name()
            .and_then(|name| name.to_str())
            .expect("file name")
    ));
    write_file(&staging, contents);
    fs::rename(&staging, path).expect("replace file");
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
