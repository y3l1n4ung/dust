use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;

use crate::{
    SourceLibrary, is_generated_primary_file, load_dust_config, load_package_name,
    primary_output_path,
};

/// Finding Dust annotation names in Dart source without parsing it.
mod annotations;
use self::annotations::*;

/// Walking the lib tree and reading the import graph out of Dart directives.
mod imports;
use self::imports::*;

/// Dust packages whose runtime versions affect generated Dart output.
const DUST_RUNTIME_PACKAGES: &[(&str, &str)] = &[
    ("dust_dart", "package:dust_dart/"),
    ("dust_flutter", "package:dust_flutter/"),
    ("dust_db_sqlite3", "package:dust_db_sqlite3/"),
    ("dust_db_postgres", "package:dust_db_postgres/"),
];

/// Deduplicated set of annotation names owned by Dust plugins.
///
/// Discovery uses these names only after a library is reachable from Dust
/// imports or re-exports.
#[derive(Debug, Clone, Default)]
pub struct SupportedAnnotations {
    /// Supported annotation short names.
    names: HashSet<Box<str>>,
}

impl SupportedAnnotations {
    /// Builds a supported annotation set from plugin-owned surface names.
    pub fn new<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        names.into_iter().collect()
    }

    /// Returns `true` when the annotation name is owned by Dust.
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }
}

impl<S> FromIterator<S> for SupportedAnnotations
where
    S: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self {
            names: iter
                .into_iter()
                .map(|name| name.into().into_boxed_str())
                .collect(),
        }
    }
}

/// Libraries selected for generation plus Dust package imports found during discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryDiscovery {
    /// Candidate Dust source libraries in deterministic order.
    pub libraries: Vec<SourceLibrary>,
    /// Dust package names imported by source files under `lib/`.
    pub dust_packages: Vec<String>,
}

/// Recursively discovers candidate Dust libraries under `lib/**/*.dart`.
///
/// The scan is deterministic and only keeps source files that:
/// - are not already generated primary output files
/// - use a supported Dust annotation
/// - import Dust directly or through local Dart import/export chains
pub fn discover_libraries(
    root: &Path,
    supported_annotations: &SupportedAnnotations,
) -> Result<Vec<SourceLibrary>, Diagnostic> {
    Ok(discover_libraries_with_usage(root, supported_annotations)?.libraries)
}

/// Discovers candidate Dust libraries and package imports in one source scan.
pub fn discover_libraries_with_usage(
    root: &Path,
    supported_annotations: &SupportedAnnotations,
) -> Result<LibraryDiscovery, Diagnostic> {
    let dust_config = load_dust_config(root)?;
    let package_name = load_package_name(root)?;
    let lib_dir = root.join("lib");
    if !lib_dir.is_dir() {
        return Ok(LibraryDiscovery {
            libraries: Vec::new(),
            dust_packages: Vec::new(),
        });
    }

    let mut dart_files = Vec::new();
    collect_dart_files(&lib_dir, &mut dart_files)?;
    dart_files.sort();

    let candidates = candidate_files(
        root,
        &lib_dir,
        &package_name,
        dart_files
            .into_iter()
            .filter(|path| !is_generated_primary_file(path, &dust_config.outputs.primary_suffix)),
        supported_annotations,
    )?;
    let dust_packages = imported_dust_packages(&candidates);
    let dust_aware = dust_aware_files(&candidates);

    let mut libraries = Vec::new();
    for source_path in candidates
        .iter()
        .filter(|candidate| {
            candidate.has_supported_annotation && dust_aware.contains(candidate.path.as_path())
        })
        .map(|candidate| candidate.path.clone())
    {
        if source_path.starts_with(root) {
            libraries.push(SourceLibrary {
                output_path: primary_output_path(&source_path, &dust_config.outputs.primary_suffix),
                source_path,
            });
        }
    }

    Ok(LibraryDiscovery {
        libraries,
        dust_packages,
    })
}

/// One Dart source file considered during discovery.
struct CandidateFile {
    /// Absolute source path.
    path: PathBuf,
    /// Whether the file directly imports a Dust package.
    direct_dust: bool,
    /// Dust runtime packages imported by this file.
    dust_packages: Vec<&'static str>,
    /// Local files imported or exported by this file.
    local_imports: Vec<PathBuf>,
    /// Whether the file contains a supported annotation name.
    has_supported_annotation: bool,
}

/// Reads candidate files and extracts discovery facts.
fn candidate_files<I>(
    root: &Path,
    lib_dir: &Path,
    package_name: &str,
    paths: I,
    supported_annotations: &SupportedAnnotations,
) -> Result<Vec<CandidateFile>, Diagnostic>
where
    I: IntoIterator<Item = PathBuf>,
{
    paths
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).map_err(|error| {
                Diagnostic::error(format!(
                    "failed to read library `{}`: {error}",
                    path.display()
                ))
            })?;
            Ok(CandidateFile {
                direct_dust: contains_dust_annotation_package(&source),
                dust_packages: dust_package_imports(&source),
                local_imports: local_imports(root, lib_dir, package_name, &path, &source),
                has_supported_annotation: has_supported_annotation(&source, supported_annotations),
                path,
            })
        })
        .collect()
}

/// Returns the deterministic set of Dust package imports found during discovery.
fn imported_dust_packages(candidates: &[CandidateFile]) -> Vec<String> {
    candidates
        .iter()
        .flat_map(|candidate| candidate.dust_packages.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Computes files reachable from direct Dust imports or local re-exports.
fn dust_aware_files(candidates: &[CandidateFile]) -> HashSet<&Path> {
    let known_paths = candidates
        .iter()
        .map(|candidate| (candidate.path.as_path(), candidate))
        .collect::<HashMap<_, _>>();
    let mut dust_aware = candidates
        .iter()
        .filter(|candidate| candidate.direct_dust)
        .map(|candidate| candidate.path.as_path())
        .collect::<HashSet<_>>();

    let mut changed = true;
    while changed {
        changed = false;
        for candidate in candidates {
            if dust_aware.contains(candidate.path.as_path()) {
                continue;
            }
            if candidate.local_imports.iter().any(|import| {
                known_paths.contains_key(import.as_path()) && dust_aware.contains(import.as_path())
            }) {
                changed = dust_aware.insert(candidate.path.as_path());
            }
        }
    }

    dust_aware
}
