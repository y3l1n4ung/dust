//! Walking the lib tree and reading the import graph out of Dart directives.

use super::*;

/// Recursively collects Dart files under one directory.
pub(super) fn collect_dart_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Diagnostic> {
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
pub(super) fn local_imports(
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
pub(super) fn directive_uris(source: &str) -> Vec<String> {
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
pub(super) fn quoted_uri_value(source: &str, quote: char) -> Option<(String, usize)> {
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
pub(super) fn resolve_local_import(
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
pub(super) fn normalize_path(path: &Path) -> PathBuf {
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
