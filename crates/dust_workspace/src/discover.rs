use std::{
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;

use crate::SourceLibrary;

/// Recursively discovers candidate Dust libraries under `lib/**/*.dart`.
///
/// The scan is deterministic and only keeps source files that:
/// - are not already generated `*.g.dart` files
/// - declare a matching `part 'x.g.dart';` or `part "x.g.dart";`
/// - contain at least one annotation marker like `@Derive` or `@Debug`
pub fn discover_libraries(root: &Path) -> Result<Vec<SourceLibrary>, Diagnostic> {
    let lib_dir = root.join("lib");
    if !lib_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut dart_files = Vec::new();
    collect_dart_files(&lib_dir, &mut dart_files)?;
    dart_files.sort();

    let mut libraries = Vec::new();
    for source_path in dart_files {
        if is_generated_dart_file(&source_path) {
            continue;
        }

        if is_candidate_library(&source_path)? {
            libraries.push(SourceLibrary {
                output_path: generated_output_path(&source_path),
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

fn is_generated_dart_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".g.dart"))
}

fn is_candidate_library(path: &Path) -> Result<bool, Diagnostic> {
    let source = fs::read_to_string(path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read library `{}`: {error}",
            path.display()
        ))
    })?;

    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return Ok(false);
    };

    let expected_single = format!("part '{stem}.g.dart';");
    let expected_double = format!("part \"{stem}.g.dart\";");

    Ok(
        (source.contains(&expected_single) || source.contains(&expected_double))
            && contains_annotation_marker(&source),
    )
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

fn generated_output_path(source_path: &Path) -> PathBuf {
    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("file");
    source_path.with_file_name(format!("{stem}.g.dart"))
}
