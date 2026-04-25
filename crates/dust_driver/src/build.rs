use std::{
    fs, io,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::Instant,
};

use dust_cache::{CacheEntry, WorkspaceCache};
use dust_diagnostics::Diagnostic;
use dust_emitter::write_library;
use dust_parser_dart::{ParseOptions, parse_file_with_backend};
use dust_parser_dart_ts::TreeSitterDartBackend;
use dust_plugin_api::PluginRegistry;
use dust_plugin_derive::register_plugin as register_derive_plugin;
use dust_plugin_serde::register_plugin as register_serde_plugin;
use dust_resolver::resolve_library;
use dust_text::{FileId, SourceText};
use dust_workspace::{SourceLibrary, discover_workspace};

use crate::{
    catalog::build_symbol_catalog,
    lower::lower_library,
    progress::{ProgressEvent, ProgressPhase},
    request::BuildRequest,
    result::{BuildArtifact, CacheReport, CommandResult},
};

const CODEGEN_FINGERPRINT_INPUT: &str = concat!(
    include_str!("build.rs"),
    include_str!("check.rs"),
    include_str!("watch.rs"),
    include_str!("lower.rs"),
    include_str!("../../dust_plugin_derive/src/plugin.rs"),
    include_str!("../../dust_plugin_derive/src/features/debug.rs"),
    include_str!("../../dust_plugin_derive/src/features/eq_hash.rs"),
    include_str!("../../dust_plugin_derive/src/features/clone_copy_with.rs"),
    include_str!("../../dust_plugin_serde/src/plugin.rs"),
    include_str!("../../dust_plugin_serde/src/validate.rs"),
    include_str!("../../dust_plugin_serde/src/emit.rs"),
    include_str!("../../dust_plugin_serde/src/writer.rs"),
    include_str!("../../dust_emitter/src/emit.rs"),
    include_str!("../../dust_emitter/src/writer.rs"),
);

/// Runs one writing build across the discovered workspace.
pub fn run_build(request: BuildRequest) -> CommandResult {
    run_build_inner(request, None)
}

/// Runs one writing build across the discovered workspace while emitting progress events.
pub fn run_build_with_progress<F>(request: BuildRequest, progress: F) -> CommandResult
where
    F: Fn(ProgressEvent) + Send + Sync,
{
    run_build_inner(request, Some(&progress))
}

fn run_build_inner(
    request: BuildRequest,
    progress: Option<&(dyn Fn(ProgressEvent) + Send + Sync)>,
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
    let package_config_hash = match read_package_config_hash(&workspace.package_config.path) {
        Ok(hash) => hash,
        Err(diagnostic) => {
            result.diagnostics.push(diagnostic);
            result.elapsed_ms = started.elapsed().as_millis();
            return result;
        }
    };
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
    let indexed = prepare_and_process_batch(
        &workspace.root,
        &workspace.libraries,
        package_config_hash,
        tool_hash,
        &cache,
        &catalog,
        &registry,
        true,
        request.fail_fast,
        request.jobs,
        1,
        ProgressPhase::Build,
        progress,
        &mut cache_report,
    );

    for indexed_outcome in indexed {
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
            break;
        }
    }

    if let Err(error) = cache.flush() {
        result.diagnostics.push(Diagnostic::error(format!(
            "failed to persist Dust cache `{}`: {error}",
            cache.path().display()
        )));
    }
    result.cache = Some(cache_report);
    result.elapsed_ms = started.elapsed().as_millis();
    result
}

pub(crate) struct BuildOutcome {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) artifact: BuildArtifact,
    pub(crate) expected_output_hash: Option<u64>,
}

pub(crate) struct IndexedBuildOutcome {
    pub(crate) index: usize,
    pub(crate) library: SourceLibrary,
    pub(crate) source_hash: Option<u64>,
    pub(crate) outcome: BuildOutcome,
}

pub(crate) struct LoadedLibraryInput {
    pub(crate) source: String,
    pub(crate) source_hash: u64,
    pub(crate) output_hash: Option<u64>,
}

struct PendingLibrary {
    index: usize,
    file_id: FileId,
    library: SourceLibrary,
    input: LoadedLibraryInput,
}

pub(crate) fn prepare_and_process_batch(
    workspace_root: &Path,
    libraries: &[SourceLibrary],
    package_config_hash: u64,
    tool_hash: u64,
    cache: &WorkspaceCache,
    catalog: &dust_resolver::SymbolCatalog,
    registry: &PluginRegistry,
    write_output: bool,
    fail_fast: bool,
    jobs: Option<usize>,
    file_id_base: u32,
    phase: ProgressPhase,
    progress: Option<&(dyn Fn(ProgressEvent) + Send + Sync)>,
    cache_report: &mut CacheReport,
) -> Vec<IndexedBuildOutcome> {
    if let Some(progress) = progress {
        progress(ProgressEvent::StartedBatch {
            phase,
            total: libraries.len(),
        });
    }

    let mut outcomes = Vec::new();
    let mut pending = Vec::new();
    let completed = Arc::new(AtomicUsize::new(0));

    for (index, library) in libraries.iter().enumerate() {
        let input = match load_library_input(library) {
            Ok(input) => input,
            Err(diagnostic) => {
                outcomes.push(IndexedBuildOutcome {
                    index,
                    library: library.clone(),
                    source_hash: None,
                    outcome: BuildOutcome {
                        diagnostics: vec![diagnostic],
                        artifact: BuildArtifact {
                            source_path: library.source_path.clone(),
                            output_path: library.output_path.clone(),
                            changed: false,
                            written: false,
                            cached: false,
                        },
                        expected_output_hash: None,
                    },
                });
                emit_progress(
                    progress,
                    &completed,
                    phase,
                    libraries.len(),
                    library,
                    false,
                    false,
                    false,
                    true,
                    0,
                );
                if fail_fast {
                    break;
                }
                continue;
            }
        };

        if matches_cache(
            cache,
            workspace_root,
            library,
            &input,
            package_config_hash,
            tool_hash,
        ) {
            cache_report.hits += 1;
            outcomes.push(IndexedBuildOutcome {
                index,
                library: library.clone(),
                source_hash: Some(input.source_hash),
                outcome: BuildOutcome {
                    diagnostics: Vec::new(),
                    artifact: BuildArtifact {
                        source_path: library.source_path.clone(),
                        output_path: library.output_path.clone(),
                        changed: false,
                        written: false,
                        cached: true,
                    },
                    expected_output_hash: input.output_hash,
                },
            });
            emit_progress(
                progress,
                &completed,
                phase,
                libraries.len(),
                library,
                true,
                false,
                false,
                false,
                0,
            );
            continue;
        }

        cache_report.misses += 1;
        pending.push(PendingLibrary {
            index,
            file_id: FileId::new(file_id_base + index as u32),
            library: library.clone(),
            input,
        });
    }

    let desired_jobs = jobs.unwrap_or_else(default_parallel_jobs).max(1);
    let mut processed = if fail_fast || desired_jobs <= 1 || pending.len() <= 1 {
        let mut processed = Vec::new();

        for pending in pending {
            let outcome = process_pending_library(
                pending,
                catalog,
                registry,
                write_output,
                phase,
                progress,
                libraries.len(),
                &completed,
            );
            let has_error = outcome
                .outcome
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.is_error());
            processed.push(outcome);

            if fail_fast && has_error {
                break;
            }
        }

        processed
    } else {
        process_pending_parallel(
            pending,
            desired_jobs,
            catalog,
            registry,
            write_output,
            phase,
            progress,
            libraries.len(),
            &completed,
        )
    };

    outcomes.append(&mut processed);
    outcomes.sort_by_key(|outcome| outcome.index);
    outcomes
}

fn process_pending_library(
    pending: PendingLibrary,
    catalog: &dust_resolver::SymbolCatalog,
    registry: &PluginRegistry,
    write_output: bool,
    phase: ProgressPhase,
    progress: Option<&(dyn Fn(ProgressEvent) + Send + Sync)>,
    total: usize,
    completed: &AtomicUsize,
) -> IndexedBuildOutcome {
    let backend = TreeSitterDartBackend::new();
    let started = Instant::now();
    let outcome = process_library_from_source(
        pending.file_id,
        &pending.library,
        pending.input.source,
        &backend,
        catalog,
        registry,
        write_output,
    );
    let elapsed_ms = started.elapsed().as_millis();
    let had_errors = outcome
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.is_error());
    emit_progress(
        progress,
        completed,
        phase,
        total,
        &pending.library,
        false,
        outcome.artifact.written,
        outcome.artifact.changed,
        had_errors,
        elapsed_ms,
    );

    IndexedBuildOutcome {
        index: pending.index,
        library: pending.library,
        source_hash: Some(pending.input.source_hash),
        outcome,
    }
}

fn process_pending_parallel(
    pending: Vec<PendingLibrary>,
    jobs: usize,
    catalog: &dust_resolver::SymbolCatalog,
    registry: &PluginRegistry,
    write_output: bool,
    phase: ProgressPhase,
    progress: Option<&(dyn Fn(ProgressEvent) + Send + Sync)>,
    total: usize,
    completed: &AtomicUsize,
) -> Vec<IndexedBuildOutcome> {
    let threads = jobs.min(pending.len()).max(1);
    let mut groups = (0..threads).map(|_| Vec::new()).collect::<Vec<_>>();
    for (index, item) in pending.into_iter().enumerate() {
        groups[index % threads].push(item);
    }

    thread::scope(|scope| {
        let mut handles = Vec::new();
        for group in groups {
            handles.push(scope.spawn(move || {
                group
                    .into_iter()
                    .map(|pending| {
                        process_pending_library(
                            pending,
                            catalog,
                            registry,
                            write_output,
                            phase,
                            progress,
                            total,
                            completed,
                        )
                    })
                    .collect::<Vec<_>>()
            }));
        }

        let mut processed = Vec::new();
        for handle in handles {
            processed.extend(handle.join().expect("worker thread must not panic"));
        }
        processed
    })
}

fn emit_progress(
    progress: Option<&(dyn Fn(ProgressEvent) + Send + Sync)>,
    completed: &AtomicUsize,
    phase: ProgressPhase,
    total: usize,
    library: &SourceLibrary,
    cached: bool,
    written: bool,
    changed: bool,
    had_errors: bool,
    elapsed_ms: u128,
) {
    if let Some(progress) = progress {
        let completed = completed.fetch_add(1, Ordering::SeqCst) + 1;
        progress(ProgressEvent::FinishedLibrary {
            phase,
            completed,
            total,
            source_path: library.source_path.clone(),
            cached,
            written,
            changed,
            had_errors,
            elapsed_ms,
        });
    }
}

fn default_parallel_jobs() -> usize {
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
}

pub(crate) fn process_library_from_source(
    file_id: FileId,
    library: &SourceLibrary,
    source: String,
    backend: &TreeSitterDartBackend,
    catalog: &dust_resolver::SymbolCatalog,
    registry: &PluginRegistry,
    write_output: bool,
) -> BuildOutcome {
    let mut diagnostics = Vec::new();
    let artifact = BuildArtifact {
        source_path: library.source_path.clone(),
        output_path: library.output_path.clone(),
        changed: false,
        written: false,
        cached: false,
    };
    let source_text = SourceText::new(file_id, source);
    let parsed = parse_file_with_backend(backend, &source_text, ParseOptions::default());
    diagnostics.extend(parsed.diagnostics.clone());

    let resolved = resolve_library(
        file_id,
        &library.source_path.to_string_lossy(),
        &parsed.library,
        catalog,
    );
    diagnostics.extend(resolved.diagnostics.clone());

    let lowered = lower_library(&resolved.library);
    diagnostics.extend(lowered.diagnostics.clone());

    if diagnostics.iter().any(|diagnostic| diagnostic.is_error()) {
        return BuildOutcome {
            diagnostics,
            artifact,
            expected_output_hash: None,
        };
    }

    let output = if write_output {
        match write_library(&lowered.value, registry) {
            Ok(output) => output,
            Err(error) => {
                diagnostics.push(Diagnostic::error(format!(
                    "failed to write `{}`: {error}",
                    library.output_path.display()
                )));
                return BuildOutcome {
                    diagnostics,
                    artifact,
                    expected_output_hash: None,
                };
            }
        }
    } else {
        let previous = fs::read_to_string(&library.output_path).ok();
        let emitted = dust_emitter::emit_library(&lowered.value, registry, previous.as_deref());
        dust_emitter::WriteResult {
            source: emitted.source,
            symbols: emitted.symbols,
            diagnostics: emitted.diagnostics,
            changed: emitted.changed,
            written: false,
            output_path: library.output_path.clone(),
        }
    };

    diagnostics.extend(output.diagnostics.clone());

    BuildOutcome {
        diagnostics,
        artifact: BuildArtifact {
            source_path: library.source_path.clone(),
            output_path: library.output_path.clone(),
            changed: output.changed,
            written: output.written,
            cached: false,
        },
        expected_output_hash: Some(hash_text(&output.source)),
    }
}

pub(crate) fn load_library_input(
    library: &SourceLibrary,
) -> Result<LoadedLibraryInput, Diagnostic> {
    let source = fs::read_to_string(&library.source_path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read `{}`: {error}",
            library.source_path.display()
        ))
    })?;
    let output_hash = read_optional_hash(&library.output_path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read `{}`: {error}",
            library.output_path.display()
        ))
    })?;

    Ok(LoadedLibraryInput {
        source_hash: hash_text(&source),
        source,
        output_hash,
    })
}

pub(crate) fn read_package_config_hash(path: &Path) -> Result<u64, Diagnostic> {
    let source = fs::read_to_string(path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read package configuration `{}`: {error}",
            path.display()
        ))
    })?;
    Ok(hash_text(&source))
}

pub(crate) fn hash_text(text: &str) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

pub(crate) fn matches_cache(
    cache: &WorkspaceCache,
    root: &Path,
    library: &SourceLibrary,
    input: &LoadedLibraryInput,
    package_config_hash: u64,
    tool_hash: u64,
) -> bool {
    let Some(entry) = cache.get(root, &library.source_path) else {
        return false;
    };

    entry.source_hash == input.source_hash
        && entry.package_config_hash == package_config_hash
        && entry.tool_hash == tool_hash
        && input.output_hash == Some(entry.expected_output_hash)
}

pub(crate) fn codegen_tool_hash() -> u64 {
    hash_text(CODEGEN_FINGERPRINT_INPUT)
}

fn read_optional_hash(path: &Path) -> io::Result<Option<u64>> {
    match fs::read_to_string(path) {
        Ok(source) => Ok(Some(hash_text(&source))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn default_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(register_derive_plugin()))
        .expect("derive plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_serde_plugin()))
        .expect("serde plugin symbol ownership must be valid");
    registry
}
