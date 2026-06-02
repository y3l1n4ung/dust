use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{LibraryIr, TypeIr};
use dust_plugin_api::SymbolPlan;

use crate::plugin::{constants::STATES_ANALYSIS_KEY, model::StateFact};

use super::StateFieldSpec;

pub(super) fn state_facts(plan: &SymbolPlan) -> HashMap<String, Vec<StateFieldSpec>> {
    plan.workspace_string_set(STATES_ANALYSIS_KEY)
        .unwrap_or_default()
        .iter()
        .filter_map(|value| serde_json::from_str::<StateFact>(value).ok())
        .map(|fact| {
            let fields = fact
                .fields
                .into_iter()
                .map(|field| StateFieldSpec {
                    name: field.name,
                    type_source: field.type_source,
                })
                .collect::<Vec<_>>();
            (fact.class_name, fields)
        })
        .collect()
}

pub(super) fn class_fields(
    library: &LibraryIr,
    state_facts: &HashMap<String, Vec<StateFieldSpec>>,
    class_name: &str,
) -> Vec<StateFieldSpec> {
    if let Some(class) = library
        .classes
        .iter()
        .find(|candidate| candidate.name == class_name)
    {
        let fields = class
            .fields
            .iter()
            .map(|field| StateFieldSpec {
                name: field.name.clone(),
                type_source: render_type(&field.ty),
            })
            .collect::<Vec<_>>();
        if !fields.is_empty() {
            return fields;
        }
    }
    if let Some(fields) = imported_class_fields(library, class_name) {
        return fields;
    }
    state_facts
        .get(class_name)
        .filter(|fields| !fields.is_empty())
        .cloned()
        .unwrap_or_default()
}

fn imported_class_fields(library: &LibraryIr, class_name: &str) -> Option<Vec<StateFieldSpec>> {
    library
        .imports
        .iter()
        .filter_map(|uri| resolve_import_path(library, uri))
        .find_map(|path| {
            let source = fs::read_to_string(path).ok()?;
            fields_from_source_class(&source, class_name)
        })
        .filter(|fields| !fields.is_empty())
}

fn resolve_import_path(library: &LibraryIr, uri: &str) -> Option<PathBuf> {
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

fn fields_from_source_class(source: &str, class_name: &str) -> Option<Vec<StateFieldSpec>> {
    let class_offset = source.find(&format!("class {class_name}"))?;
    let body_start = source[class_offset..].find('{')? + class_offset;
    let body_end = matching_brace(source, body_start)?;
    let body = &source[body_start + 1..body_end];
    let declared_names = declared_type_names(source);
    Some(
        body.lines()
            .filter_map(|line| field_from_line(line, &declared_names))
            .collect(),
    )
}

fn field_from_line(line: &str, declared_names: &[String]) -> Option<StateFieldSpec> {
    let mut line = line.trim();
    if line.starts_with('@') || line.starts_with("//") || line.contains('(') || !line.ends_with(';')
    {
        return None;
    }
    line = line.trim_end_matches(';').trim();
    let declaration = line.split('=').next().unwrap_or(line).trim();
    let parts = declaration.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 2
        || !parts
            .iter()
            .any(|part| matches!(*part, "final" | "var" | "late"))
    {
        return None;
    }
    let name = parts.last()?.trim_end_matches(';');
    if matches!(name, "get" | "set") {
        return None;
    }
    let type_parts = parts[..parts.len() - 1]
        .iter()
        .copied()
        .filter(|part| !matches!(*part, "static" | "late" | "final" | "var" | "const"))
        .collect::<Vec<_>>();
    if type_parts.is_empty() {
        return None;
    }
    Some(StateFieldSpec {
        name: name.to_owned(),
        type_source: sanitize_imported_type(&type_parts.join(" "), declared_names),
    })
}

fn declared_type_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line
                .strip_prefix("class ")
                .or_else(|| line.strip_prefix("final class "))
                .or_else(|| line.strip_prefix("sealed class "))
                .or_else(|| line.strip_prefix("enum "))?;
            Some(
                rest.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                    .next()
                    .unwrap_or_default()
                    .to_owned(),
            )
        })
        .filter(|name| !name.is_empty())
        .collect()
}

fn sanitize_imported_type(type_source: &str, declared_names: &[String]) -> String {
    let ty = type_source.trim();
    if let Some(inner) = ty
        .strip_prefix("List<")
        .and_then(|value| value.strip_suffix('>'))
    {
        return if is_visible_imported_type(inner.trim(), declared_names) {
            ty.to_owned()
        } else {
            "List<Object?>".to_owned()
        };
    }
    if ty.contains('<') {
        return "Object?".to_owned();
    }
    if is_visible_imported_type(ty.trim_end_matches('?'), declared_names) {
        ty.to_owned()
    } else {
        "Object?".to_owned()
    }
}

fn is_visible_imported_type(type_name: &str, declared_names: &[String]) -> bool {
    matches!(
        type_name,
        "String" | "int" | "double" | "num" | "bool" | "DateTime" | "Object" | "dynamic" | "void"
    ) || declared_names.iter().any(|name| name == type_name)
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0_i32;
    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

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

fn render_type(ty: &TypeIr) -> String {
    DYNAMIC_TYPES.render(ty)
}
