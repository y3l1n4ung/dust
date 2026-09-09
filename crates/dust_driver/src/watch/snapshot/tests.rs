use tempfile::tempdir;

use super::*;

fn library(path: &str) -> SourceLibrary {
    let source_path = PathBuf::from(path);
    let output_path = PathBuf::from(path.replace(".dart", ".g.dart"));
    SourceLibrary {
        source_path,
        output_path,
    }
}

fn snapshot(hash: Option<u64>, libraries: Vec<(&str, u64)>) -> WorkspaceSnapshot {
    let libraries = libraries
        .into_iter()
        .map(|(path, source_hash)| {
            let library = library(path);
            (
                library.source_path.clone(),
                SnapshotEntry {
                    library,
                    source_hash,
                    contains_route_marker: false,
                    contains_router_marker: false,
                },
            )
        })
        .collect();

    WorkspaceSnapshot {
        package_config_hash: hash,
        libraries,
    }
}

#[test]
fn changed_libraries_rebuilds_all_when_package_config_hash_changes() {
    let previous = snapshot(Some(1), vec![("lib/a.dart", 10), ("lib/b.dart", 20)]);
    let next = snapshot(Some(2), vec![("lib/a.dart", 10), ("lib/b.dart", 20)]);

    let changed = changed_libraries(&previous, &next);

    assert_eq!(changed.len(), 2);
    assert_eq!(changed[0].source_path, PathBuf::from("lib/a.dart"));
    assert_eq!(changed[1].source_path, PathBuf::from("lib/b.dart"));
}

#[test]
fn changed_libraries_detects_added_and_modified_files_in_order() {
    let previous = snapshot(Some(1), vec![("lib/a.dart", 10)]);
    let next = snapshot(Some(1), vec![("lib/a.dart", 11), ("lib/b.dart", 20)]);

    let changed = changed_libraries(&previous, &next);

    assert_eq!(changed.len(), 2);
    assert_eq!(changed[0].source_path, PathBuf::from("lib/a.dart"));
    assert_eq!(changed[1].source_path, PathBuf::from("lib/b.dart"));
}

#[test]
fn changed_libraries_ignores_removed_and_unchanged_files() {
    let previous = snapshot(Some(1), vec![("lib/a.dart", 10), ("lib/old.dart", 99)]);
    let next = snapshot(Some(1), vec![("lib/a.dart", 10)]);

    let changed = changed_libraries(&previous, &next);

    assert!(changed.is_empty());
}

#[test]
fn route_changes_also_rebuild_router_roots() {
    let route_page = SourceLibrary {
        source_path: PathBuf::from("lib/pages/project_page.dart"),
        output_path: PathBuf::from("lib/pages/project_page.g.dart"),
    };
    let router = SourceLibrary {
        source_path: PathBuf::from("lib/route.dart"),
        output_path: PathBuf::from("lib/route.g.dart"),
    };
    let not_found_page = SourceLibrary {
        source_path: PathBuf::from("lib/pages/not_found_page.dart"),
        output_path: PathBuf::from("lib/pages/not_found_page.g.dart"),
    };
    let snapshot = WorkspaceSnapshot {
        package_config_hash: Some(1),
        libraries: BTreeMap::from([
            (
                route_page.source_path.clone(),
                SnapshotEntry {
                    library: route_page.clone(),
                    source_hash: 2,
                    contains_route_marker: true,
                    contains_router_marker: false,
                },
            ),
            (
                not_found_page.source_path.clone(),
                SnapshotEntry {
                    library: not_found_page,
                    source_hash: 1,
                    contains_route_marker: true,
                    contains_router_marker: false,
                },
            ),
            (
                router.source_path.clone(),
                SnapshotEntry {
                    library: router.clone(),
                    source_hash: 1,
                    contains_route_marker: false,
                    contains_router_marker: true,
                },
            ),
        ]),
    };

    let expanded = expand_route_rebuilds(vec![route_page.clone()], &snapshot);

    assert_eq!(
        expanded
            .into_iter()
            .map(|library| library.source_path)
            .collect::<Vec<_>>(),
        vec![
            PathBuf::from("lib/pages/not_found_page.dart"),
            PathBuf::from("lib/pages/project_page.dart"),
            PathBuf::from("lib/route.dart"),
        ]
    );
}

#[test]
fn build_snapshot_hashes_package_config_and_library_contents() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let dart_tool = root.join(".dart_tool");
    let lib = root.join("lib");
    fs::create_dir_all(&dart_tool).unwrap();
    fs::create_dir_all(&lib).unwrap();
    fs::write(root.join("pubspec.yaml"), "name: sample\n").unwrap();
    fs::write(
        dart_tool.join("package_config.json"),
        r#"{"configVersion":2,"packages":[]}"#,
    )
    .unwrap();
    fs::write(
            lib.join("user.dart"),
            "import 'package:dust_dart/derive.dart';\npart 'user.g.dart';\n@Derive([ToString()])\nclass User with _$User { const User(); }\n",
        )
        .unwrap();

    let snapshot = build_snapshot(root).unwrap();

    assert!(snapshot.package_config_hash.is_some());
    assert_eq!(snapshot.libraries.len(), 1);
    assert!(snapshot.libraries.contains_key(&lib.join("user.dart")));
}

#[test]
fn build_snapshot_hash_changes_when_pubspec_changes() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let dart_tool = root.join(".dart_tool");
    let lib = root.join("lib");
    fs::create_dir_all(&dart_tool).unwrap();
    fs::create_dir_all(&lib).unwrap();
    fs::write(root.join("pubspec.yaml"), "name: sample\n").unwrap();
    fs::write(
        dart_tool.join("package_config.json"),
        r#"{"configVersion":2,"packages":[]}"#,
    )
    .unwrap();
    fs::write(
            lib.join("user.dart"),
            "import 'package:dust_dart/derive.dart';\npart 'user.g.dart';\n@Derive([Validate()])\nclass User with _$User { const User(); }\n",
        )
        .unwrap();

    let first = build_snapshot(root).unwrap().package_config_hash;
    fs::write(
        root.join("pubspec.yaml"),
        "name: sample\ndependencies:\n  flutter:\n    sdk: flutter\n",
    )
    .unwrap();
    let next = build_snapshot(root).unwrap().package_config_hash;

    assert_ne!(first, next);
}
