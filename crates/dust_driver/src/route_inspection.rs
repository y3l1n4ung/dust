use std::{path::Path, time::Instant};

use dust_plugin_api::LibraryAnalysisSnapshot;

use crate::{
    build::{BatchConfig, RegistrySelection, prepare_and_process_batch},
    context::CachedDriverContext,
    progress::ProgressPhase,
    request::DbRequestOptions,
    result::{CacheReport, CommandResult},
};

/// Route analysis snapshots collected from a read-only workspace scan.
pub(crate) struct RouteInspection {
    /// Partially populated command result with diagnostics and cache metadata.
    pub(crate) result: CommandResult,
    /// Number of Dart libraries scanned for route facts.
    pub(crate) scanned_files: usize,
    /// Analysis snapshots used by route inspection tools.
    pub(crate) snapshots: Vec<LibraryAnalysisSnapshot>,
}

/// Collects route analysis snapshots without writing generated files.
pub(crate) fn collect_route_inspection(cwd: &Path) -> RouteInspection {
    let started = Instant::now();
    let mut result = CommandResult::default();

    let CachedDriverContext {
        workspace,
        registry,
        catalog,
        tool_hash,
        package_config_hash,
        cache,
        mut cache_report,
    } = match CachedDriverContext::load(
        cwd,
        RegistrySelection::for_check(DbRequestOptions::default()),
    ) {
        Ok(context) => context,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return RouteInspection {
                result,
                scanned_files: 0,
                snapshots: Vec::new(),
            };
        }
    };

    let indexed = prepare_and_process_batch(
        BatchConfig {
            cache_root: &workspace.cache_root,
            package_root: &workspace.package_root,
            package_name: &workspace.package_name,
            is_flutter_package: workspace.is_flutter_package,
            dart_language_version: workspace.dart_sdk_lower_bound.unwrap_or_default(),
            package_config_hash,
            tool_hash,
            cache: &cache,
            catalog: &catalog,
            registry: &registry,
            write_output: false,
            fail_fast: false,
            jobs: None,
            file_id_base: 1,
            phase: ProgressPhase::Build,
            progress: None,
        },
        &workspace.libraries,
        &mut cache_report,
    );

    let snapshots = indexed
        .iter()
        .map(|outcome| outcome.outcome.analysis_snapshot.clone())
        .collect::<Vec<_>>();
    result.diagnostics.extend(
        indexed
            .iter()
            .flat_map(|outcome| outcome.outcome.diagnostics.clone()),
    );
    result.diagnostic_files.extend(
        indexed
            .into_iter()
            .filter(|outcome| {
                outcome
                    .outcome
                    .diagnostics
                    .iter()
                    .any(dust_diagnostics::Diagnostic::has_labels)
            })
            .filter_map(|outcome| outcome.outcome.diagnostic_file),
    );
    result.cache = Some(CacheReport {
        path: cache_report.path,
        hits: cache_report.hits,
        misses: cache_report.misses,
    });
    result.elapsed_ms = started.elapsed().as_millis();

    RouteInspection {
        result,
        scanned_files: workspace.libraries.len(),
        snapshots,
    }
}
