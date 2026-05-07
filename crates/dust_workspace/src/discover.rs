use std::{
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;

use crate::{SourceLibrary, is_generated_primary_file, load_dust_config, primary_output_path};

/// Recursively discovers candidate Dust libraries under `lib/**/*.dart`.
///
/// The scan is deterministic and only keeps source files that:
/// - are not already generated primary output files
/// - contain at least one annotation marker like `@Derive` or `@ToString`
pub fn discover_libraries(root: &Path) -> Result<Vec<SourceLibrary>, Diagnostic> {
    let dust_config = load_dust_config(root)?;
    let lib_dir = root.join("lib");
    if !lib_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut dart_files = Vec::new();
    collect_dart_files(&lib_dir, &mut dart_files)?;
    dart_files.sort();

    let mut libraries = Vec::new();
    for source_path in dart_files {
        if is_generated_primary_file(&source_path, &dust_config.outputs.primary_suffix) {
            continue;
        }

        if is_candidate_library(&source_path)? {
            libraries.push(SourceLibrary {
                output_path: primary_output_path(&source_path, &dust_config.outputs.primary_suffix),
                source_path,
            });
        }
    }

    Ok(libraries)
}

fn collect_dart_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Diagnostic> {
    let mut entries = fs::read_dir(dir)
        .map_err(|error| {
            Diagnostic::error(format!(
                "failed to read directory `{}`: {error}",
                dir.display()
            ))
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            Diagnostic::error(format!(
                "failed to enumerate directory `{}`: {error}",
                dir.display()
            ))
        })?;

    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dart_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("dart") {
            out.push(path);
        }
    }

    Ok(())
}

fn is_candidate_library(path: &Path) -> Result<bool, Diagnostic> {
    let source = fs::read_to_string(path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read library `{}`: {error}",
            path.display()
        ))
    })?;

    Ok(contains_annotation_marker(&source))
}

fn contains_annotation_marker(source: &str) -> bool {
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '@' {
            while let Some(next) = chars.peek() {
                if next.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }

            if let Some(next) = chars.peek() {
                if *next == '_' || next.is_ascii_alphabetic() {
                    return true;
                }
            }
        }
    }

    false
}
