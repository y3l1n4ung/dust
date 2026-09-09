use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use dust_workspace::{SourceLibrary, SupportedAnnotations, discover_workspace};

use crate::build::{default_registry, hash_text, read_workspace_config_hash};

/// Snapshot of workspace inputs relevant to watch rebuild decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceSnapshot {
    /// Current package and Dust configuration hash, if readable.
    pub(crate) package_config_hash: Option<u64>,
    /// Source libraries keyed by source path.
    pub(crate) libraries: BTreeMap<PathBuf, SnapshotEntry>,
}

/// Source metadata for one library in a watch snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SnapshotEntry {
    /// Source and output paths for the library.
    pub(crate) library: SourceLibrary,
    /// Hash of the current source text.
    pub(crate) source_hash: u64,
    /// Whether the source contains a route declaration marker.
    pub(crate) contains_route_marker: bool,
    /// Whether the source contains a router declaration marker.
    pub(crate) contains_router_marker: bool,
}

/// Builds a fresh workspace snapshot from disk.
pub(crate) fn build_snapshot(cwd: &Path) -> Result<WorkspaceSnapshot, Diagnostic> {
    let registry = default_registry();
    let supported_annotations: SupportedAnnotations =
        registry.all_supported_annotations().into_iter().collect();
    let workspace = discover_workspace(cwd, &supported_annotations)?;
    let package_config_hash = read_workspace_config_hash(
        &workspace.package_config.path,
        workspace.dust_config.path.as_deref(),
        &workspace.package_root.join("pubspec.yaml"),
    )
    .ok();

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
                contains_route_marker: source.contains("@AppRoute"),
                contains_router_marker: source.contains("@AppRouter"),
            },
        );
    }

    Ok(WorkspaceSnapshot {
        package_config_hash,
        libraries,
    })
}

/// Adds route/router libraries when route declaration analysis changed.
pub(crate) fn expand_route_rebuilds(
    changed: Vec<SourceLibrary>,
    snapshot: &WorkspaceSnapshot,
) -> Vec<SourceLibrary> {
    let route_analysis_changed = changed.iter().any(|library| {
        snapshot
            .libraries
            .get(&library.source_path)
            .is_some_and(|entry| entry.contains_route_marker)
    });
    if !route_analysis_changed {
        return changed;
    }

    let mut by_path = BTreeMap::new();
    for library in changed {
        by_path.insert(library.source_path.clone(), library);
    }
    for entry in snapshot.libraries.values() {
        if entry.contains_route_marker || entry.contains_router_marker {
            by_path.insert(entry.library.source_path.clone(), entry.library.clone());
        }
    }
    by_path.into_values().collect()
}

/// Computes changed libraries between two workspace snapshots.
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
#[path = "snapshot/tests.rs"]
mod tests;
