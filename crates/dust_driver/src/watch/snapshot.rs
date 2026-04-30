use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use dust_workspace::{SourceLibrary, discover_workspace};

use crate::build::{hash_text, read_package_config_hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceSnapshot {
    pub(crate) package_config_hash: Option<u64>,
    pub(crate) libraries: BTreeMap<PathBuf, SnapshotEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SnapshotEntry {
    pub(crate) library: SourceLibrary,
    pub(crate) source_hash: u64,
}

pub(crate) fn build_snapshot(cwd: &Path) -> Result<WorkspaceSnapshot, Diagnostic> {
    let workspace = discover_workspace(cwd)?;
    let package_config_hash = read_package_config_hash(&workspace.package_config.path).ok();

    let mut libraries = BTreeMap::new();
    for library in workspace.libraries {
        let source = fs::read_to_string(&library.source_path).map_err(|error| {
            Diagnostic::error(format!(
                "failed to read `{}` during watch scan: {error}",
                library.source_path.display()
            ))
        })?;
        libraries.insert(
            library.source_path.clone(),
            SnapshotEntry {
                library,
                source_hash: hash_text(&source),
            },
        );
    }

    Ok(WorkspaceSnapshot {
        package_config_hash,
        libraries,
    })
}

pub(crate) fn changed_libraries(
    previous: &WorkspaceSnapshot,
    next: &WorkspaceSnapshot,
) -> Vec<SourceLibrary> {
    let mut changed = Vec::new();
    let rebuild_all = previous.package_config_hash != next.package_config_hash;

    if rebuild_all {
        changed.extend(next.libraries.values().map(|entry| entry.library.clone()));
    } else {
        let mut paths = BTreeSet::new();
        paths.extend(previous.libraries.keys().cloned());
        paths.extend(next.libraries.keys().cloned());

        for path in paths {
            match (previous.libraries.get(&path), next.libraries.get(&path)) {
                (None, Some(entry)) => changed.push(entry.library.clone()),
                (Some(previous), Some(next)) if previous.source_hash != next.source_hash => {
                    changed.push(next.library.clone())
                }
                _ => {}
            }
        }
    }

    changed.sort_by_key(|library| library.source_path.clone());
    changed
}

#[cfg(test)]
mod tests {
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
            "import 'package:derive_annotation/derive_annotation.dart';\npart 'user.g.dart';\n@Derive([ToString()])\nclass User with _$UserDust { const User(); }\n",
        )
        .unwrap();

        let snapshot = build_snapshot(root).unwrap();

        assert!(snapshot.package_config_hash.is_some());
        assert_eq!(snapshot.libraries.len(), 1);
        assert!(snapshot.libraries.contains_key(&lib.join("user.dart")));
    }
}
