use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;

use crate::{
    SourceLibrary, is_generated_primary_file, load_dust_config, load_package_name,
    primary_output_path,
};

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
    let dust_config = load_dust_config(root)?;
    let package_name = load_package_name(root)?;
    let lib_dir = root.join("lib");
    if !lib_dir.is_dir() {
        return Ok(Vec::new());
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

    Ok(libraries)
}

/// One Dart source file considered during discovery.
struct CandidateFile {
    /// Absolute source path.
    path: PathBuf,
    /// Whether the file directly imports a Dust package.
    direct_dust: bool,
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
                direct_dust: contains_dust_package(&source),
                local_imports: local_imports(root, lib_dir, package_name, &path, &source),
                has_supported_annotation: has_supported_annotation(&source, supported_annotations),
                path,
            })
        })
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

/// Recursively collects Dart files under one directory.
fn collect_dart_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Diagnostic> {
    let entries = fs::read_dir(dir).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read directory `{}`: {error}",
            dir.display()
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            Diagnostic::error(format!(
                "failed to enumerate directory `{}`: {error}",
                dir.display()
            ))
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            Diagnostic::error(format!(
                "failed to inspect directory entry `{}`: {error}",
                path.display()
            ))
        })?;

        if file_type.is_dir() {
            collect_dart_files(&path, out)?;
        } else if file_type.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("dart")
        {
            out.push(path);
        }
    }

    Ok(())
}

/// Resolves local import and export directives for one source file.
fn local_imports(
    root: &Path,
    lib_dir: &Path,
    package_name: &str,
    path: &Path,
    source: &str,
) -> Vec<PathBuf> {
    directive_uris(source)
        .into_iter()
        .filter_map(|uri| resolve_local_import(root, lib_dir, package_name, path, &uri))
        .collect()
}

/// Extracts import and export URI strings from Dart source.
fn directive_uris(source: &str) -> Vec<String> {
    let mut uris = Vec::new();
    for directive in ["import", "export"] {
        let mut rest = source;
        while let Some(index) = rest.find(directive) {
            rest = &rest[index + directive.len()..];
            let Some((quote_index, quote)) =
                rest.char_indices().find(|(_, ch)| matches!(ch, '\'' | '"'))
            else {
                break;
            };
            let value_start = quote_index + quote.len_utf8();
            let Some((uri, value_end)) = quoted_uri_value(&rest[value_start..], quote) else {
                break;
            };
            uris.push(uri);
            rest = &rest[value_start + value_end + quote.len_utf8()..];
        }
    }
    uris
}

/// Reads a quoted URI value and returns it with its closing quote offset.
fn quoted_uri_value(source: &str, quote: char) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    for (index, ch) in source.char_indices() {
        if escaped {
            if ch == quote || ch == '\\' {
                value.push(ch);
            } else {
                value.push('\\');
                value.push(ch);
            }
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some((value, index));
        }
        value.push(ch);
    }
    None
}

/// Resolves one Dart import URI to a local file path when possible.
fn resolve_local_import(
    root: &Path,
    lib_dir: &Path,
    package_name: &str,
    path: &Path,
    uri: &str,
) -> Option<PathBuf> {
    if uri.starts_with("dart:") {
        return None;
    }
    if let Some(package_path) = uri.strip_prefix(&format!("package:{package_name}/")) {
        return Some(normalize_path(&lib_dir.join(package_path)));
    }
    if uri.starts_with("package:") || !uri.ends_with(".dart") {
        return None;
    }
    let parent = path.parent().unwrap_or(root);
    Some(normalize_path(&parent.join(uri)))
}

/// Normalizes `.` and `..` path components without touching the filesystem.
fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

/// Returns whether source contains any supported annotation short name.
fn has_supported_annotation(source: &str, supported_annotations: &SupportedAnnotations) -> bool {
    annotation_short_names(source).any(|name| supported_annotations.contains(name))
}

/// Iterates annotation short names, including names from prefixed annotations.
fn annotation_short_names(source: &str) -> impl Iterator<Item = &str> {
    source.match_indices('@').filter_map(|(index, _)| {
        let mut start = index + 1;
        let bytes = source.as_bytes();
        while bytes.get(start).is_some_and(u8::is_ascii_whitespace) {
            start += 1;
        }
        let mut end = start;
        while bytes
            .get(end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.'))
        {
            end += 1;
        }
        source[start..end]
            .rsplit('.')
            .next()
            .filter(|name| !name.is_empty())
    })
}

/// Returns whether source imports or exports a Dust package URI.
fn contains_dust_package(source: &str) -> bool {
    source.contains("package:dust_dart/") || source.contains("package:dust_flutter/")
}
