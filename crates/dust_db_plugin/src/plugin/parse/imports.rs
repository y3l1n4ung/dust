use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use dust_ir::DartFileIr;

/// Returns row class names discovered through source library imports.
pub(crate) fn imported_row_names(library: &DartFileIr) -> HashSet<String> {
    library
        .imports
        .iter()
        .filter_map(|uri| resolve_import_path(library, uri))
        .filter_map(|path| fs::read_to_string(path).ok())
        .flat_map(|source| row_names_from_source(&source))
        .collect()
}

/// Resolves a Dart import URI to a local source path when possible.
fn resolve_import_path(library: &DartFileIr, uri: &str) -> Option<PathBuf> {
    if uri.starts_with("dart:") || uri.starts_with("package:flutter/") {
        return None;
    }
    if let Some(rest) = uri.strip_prefix("package:") {
        let (package, path) = rest.split_once('/')?;
        if package == library.package_name {
            return Some(Path::new(&library.package_root).join("lib").join(path));
        }
        return None;
    }
    let source_dir = Path::new(&library.package_root)
        .join(&library.source_path)
        .parent()?
        .to_path_buf();
    Some(normalize_path(&source_dir.join(uri)))
}

/// Normalizes a path by resolving current and parent components.
fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Scans source text for classes annotated with `FromRow`.
fn row_names_from_source(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut metadata = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('@') {
            metadata.push_str(trimmed);
            continue;
        }
        if let Some(name) = class_name_from_line(trimmed) {
            if metadata.contains("FromRow") {
                names.push(name.to_owned());
            }
            metadata.clear();
            continue;
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            metadata.clear();
        }
    }
    names
}

/// Extracts a class name from a simple Dart class declaration line.
fn class_name_from_line(line: &str) -> Option<&str> {
    let rest = line
        .strip_prefix("class ")
        .or_else(|| line.strip_prefix("final class "))
        .or_else(|| line.strip_prefix("abstract class "))
        .or_else(|| line.strip_prefix("abstract final class "))?;
    rest.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .next()
        .filter(|name| !name.is_empty())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use dust_ir::{DartFileIr, SpanIr};
    use dust_text::{FileId, TextRange};

    use super::*;

    fn span() -> SpanIr {
        SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("dust_db_parse_{name}_{stamp}"))
    }

    #[test]
    fn imported_row_names_scan_package_and_relative_imports() {
        let root = temp_root("imports");
        fs::create_dir_all(root.join("lib/models")).unwrap();
        fs::create_dir_all(root.join("lib/dao")).unwrap();
        fs::write(
            root.join("lib/models/user_row.dart"),
            "@Derive([FromRow()])\nfinal class UserRow {}\n",
        )
        .unwrap();
        fs::write(
            root.join("lib/models/team_row.dart"),
            "@FromRow()\nabstract final class TeamRow {}\n",
        )
        .unwrap();
        let library = DartFileIr {
            package_root: root.display().to_string(),
            package_name: "example".to_owned(),
            source_path: "lib/dao/user_dao.dart".to_owned(),
            output_path: "lib/dao/user_dao.g.dart".to_owned(),
            imports: vec![
                "../models/user_row.dart".to_owned(),
                "package:example/models/team_row.dart".to_owned(),
                "package:other/ignored.dart".to_owned(),
                "dart:async".to_owned(),
                "package:flutter/widgets.dart".to_owned(),
            ],
            library: None,
            library_annotations: Vec::new(),
            import_directives: Vec::new(),
            export_directives: Vec::new(),
            part_directives: Vec::new(),
            part_of: None,
            span: span(),
            classes: Vec::new(),
            mixins: Vec::new(),
            extensions: Vec::new(),
            extension_types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
            enums: Vec::new(),
            query_calls: Vec::new(),
        };

        let names = imported_row_names(&library);
        assert!(names.contains("UserRow"));
        assert!(names.contains("TeamRow"));
        assert_eq!(names.len(), 2);

        let _ = fs::remove_dir_all(root);
    }
}
