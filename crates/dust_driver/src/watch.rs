use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use dust_cache::{CacheEntry, WorkspaceCache};
use dust_diagnostics::Diagnostic;
use dust_workspace::{SourceLibrary, discover_workspace};

use crate::{
    build::{
        BatchConfig, codegen_tool_hash, default_registry, hash_text, prepare_and_process_batch,
        read_package_config_hash,
    },
    catalog::build_symbol_catalog,
    progress::{ProgressEvent, ProgressPhase},
    request::WatchRequest,
    result::{CacheReport, CommandResult, WatchReport},
};

/// Runs initial build plus repeated poll-based rebuild detection.
pub fn run_watch(request: WatchRequest) -> CommandResult {
    run_watch_inner(request, None)
}

/// Runs initial build plus repeated poll-based rebuild detection while emitting progress events.
pub fn run_watch_with_progress<F>(request: WatchRequest, progress: F) -> CommandResult
where
    F: Fn(ProgressEvent) + Send + Sync,
{
    run_watch_inner(request, Some(&progress))
}

fn run_watch_inner(
    request: WatchRequest,
    progress: Option<&(dyn Fn(ProgressEvent) + Send + Sync + '_)>,
) -> CommandResult {
    let started = Instant::now();
    let mut result = CommandResult::default();

    let workspace = match discover_workspace(&request.cwd) {
        Ok(workspace) => workspace,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };

    let registry = default_registry();
    let catalog = build_symbol_catalog(&registry);
    let tool_hash = codegen_tool_hash();
    let mut cache = match WorkspaceCache::load(&workspace.root) {
        Ok(cache) => cache,
        Err(error) => {
            result.diagnostics.push(Diagnostic::error(format!(
                "failed to load Dust cache `{}`: {error}",
                workspace
                    .root
                    .join(".dart_tool/dust/build_cache_v1.json")
                    .display()
            )));
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };
    let mut cache_report = CacheReport {
        path: cache.path().to_path_buf(),
        ..CacheReport::default()
    };
    let mut package_config_hash = match read_package_config_hash(&workspace.package_config.path) {
        Ok(hash) => hash,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };

    let initial = prepare_and_process_batch(
        BatchConfig {
            workspace_root: &workspace.root,
            package_config_hash,
            tool_hash,
            cache: &cache,
            catalog: &catalog,
            registry: &registry,
            write_output: true,
            fail_fast: request.fail_fast,
            jobs: request.jobs,
            file_id_base: 1,
            phase: ProgressPhase::WatchInitial,
            progress,
        },
        &workspace.libraries,
        &mut cache_report,
    );

    for indexed_outcome in initial {
        let has_error = indexed_outcome
            .outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error());
        if let Some(expected_output_hash) = indexed_outcome.outcome.expected_output_hash {
            if let Some(source_hash) = indexed_outcome.source_hash {
                cache.insert(
                    &workspace.root,
                    &indexed_outcome.library.source_path,
                    CacheEntry {
                        source_hash,
                        package_config_hash,
                        tool_hash,
                        expected_output_hash,
                    },
                );
            }
        } else {
            cache.remove(&workspace.root, &indexed_outcome.library.source_path);
        }
        result
            .diagnostics
            .extend(indexed_outcome.outcome.diagnostics);
        result
            .build_artifacts
            .push(indexed_outcome.outcome.artifact);

        if request.fail_fast && has_error {
            if let Err(error) = cache.flush() {
                result.diagnostics.push(Diagnostic::error(format!(
                    "failed to persist Dust cache `{}`: {error}",
                    cache.path().display()
                )));
            }
            result.watch = Some(WatchReport {
                cycles: 0,
                rebuild_batches: 0,
                rebuilt_libraries: Vec::new(),
            });
            result.cache = Some(cache_report);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    }

    let mut snapshot = match build_snapshot(&request.cwd) {
        Ok(snapshot) => snapshot,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.watch = Some(WatchReport {
                cycles: 0,
                rebuild_batches: 0,
                rebuilt_libraries: Vec::new(),
            });
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };

    let mut watch = WatchReport {
        cycles: 0,
        rebuild_batches: 0,
        rebuilt_libraries: Vec::new(),
    };
    let max_cycles = request.max_cycles.unwrap_or(u32::MAX);

    for cycle in 0..max_cycles {
        thread::sleep(Duration::from_millis(request.poll_interval_ms));
        watch.cycles = cycle + 1;

        let next_snapshot = match build_snapshot(&request.cwd) {
            Ok(snapshot) => snapshot,
            Err(diagnostic) => {
                result.diagnostics.push(diagnostic);
                break;
            }
        };

        let changed = changed_libraries(&snapshot, &next_snapshot);
        package_config_hash = next_snapshot
            .package_config_hash
            .unwrap_or(package_config_hash);
        snapshot = next_snapshot;

        if changed.is_empty() {
            continue;
        }

        watch.rebuild_batches += 1;
        let rebuilt = prepare_and_process_batch(
            BatchConfig {
                workspace_root: &workspace.root,
                package_config_hash,
                tool_hash,
                cache: &cache,
                catalog: &catalog,
                registry: &registry,
                write_output: true,
                fail_fast: request.fail_fast,
                jobs: request.jobs,
                file_id_base: 10_000,
                phase: ProgressPhase::WatchRebuild,
                progress,
            },
            &changed,
            &mut cache_report,
        );

        for indexed_outcome in rebuilt {
            let has_error = indexed_outcome
                .outcome
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.is_error());
            if let Some(expected_output_hash) = indexed_outcome.outcome.expected_output_hash {
                if let Some(source_hash) = indexed_outcome.source_hash {
                    cache.insert(
                        &workspace.root,
                        &indexed_outcome.library.source_path,
                        CacheEntry {
                            source_hash,
                            package_config_hash,
                            tool_hash,
                            expected_output_hash,
                        },
                    );
                }
            } else {
                cache.remove(&workspace.root, &indexed_outcome.library.source_path);
            }
            result
                .diagnostics
                .extend(indexed_outcome.outcome.diagnostics);
            watch
                .rebuilt_libraries
                .push(indexed_outcome.outcome.artifact.source_path.clone());
            result
                .build_artifacts
                .push(indexed_outcome.outcome.artifact);

            if request.fail_fast && has_error {
                if let Err(error) = cache.flush() {
                    result.diagnostics.push(Diagnostic::error(format!(
                        "failed to persist Dust cache `{}`: {error}",
                        cache.path().display()
                    )));
                }
                result.watch = Some(watch);
                result.cache = Some(cache_report);
                result.elapsed_ms = started.elapsed().as_millis();
                return result;
            }
        }
    }

    if let Err(error) = cache.flush() {
        result.diagnostics.push(Diagnostic::error(format!(
            "failed to persist Dust cache `{}`: {error}",
            cache.path().display()
        )));
    }
    result.watch = Some(watch);
    result.cache = Some(cache_report);
    result.elapsed_ms = started.elapsed().as_millis();
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkspaceSnapshot {
    package_config_hash: Option<u64>,
    libraries: BTreeMap<PathBuf, SnapshotEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnapshotEntry {
    library: SourceLibrary,
    source_hash: u64,
}

fn build_snapshot(cwd: &Path) -> Result<WorkspaceSnapshot, Diagnostic> {
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

fn changed_libraries(previous: &WorkspaceSnapshot, next: &WorkspaceSnapshot) -> Vec<SourceLibrary> {
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
