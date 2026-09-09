//! Finding Dust annotation names in Dart source without parsing it.

use super::*;

/// Returns whether source contains any supported annotation short name.
pub(super) fn has_supported_annotation(
    source: &str,
    supported_annotations: &SupportedAnnotations,
) -> bool {
    annotation_short_names(source)
        .into_iter()
        .any(|name| supported_annotations.contains(name))
}

/// Collects annotation short names, including the ones nested in an argument list.
///
/// The name written after the `@` is not always the one a plugin owns.
/// `@Derive([ToString(), FromRow()])` is how a row mapper is normally declared,
/// so a scan that reads `Derive` and stops leaves the file invisible to every
/// plugin whose annotation is inside the brackets — which is how a row class
/// stayed out of `dust db build` while a query was checked against it.
///
/// Inside the argument list only constructor calls count: an identifier
/// immediately followed by `(`. String literals are skipped, so the SQL handed
/// to `@Query` does not contribute `count` from `count(*)`.
pub(super) fn annotation_short_names(source: &str) -> Vec<&str> {
    let mut names = Vec::new();
    for (index, _) in source.match_indices('@') {
        let Some((name, after)) = identifier_at(source, index + 1) else {
            continue;
        };
        names.push(short_name(name));
        let after = skip_whitespace(source, after);
        if source[after..].starts_with('(') {
            collect_call_names(source, after, &mut names);
        }
    }
    names
}

/// Collects the names called inside one balanced annotation argument list.
pub(super) fn collect_call_names<'a>(source: &'a str, open: usize, names: &mut Vec<&'a str>) {
    let bytes = source.as_bytes();
    let mut depth = 0_usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => {
                depth += 1;
                index += 1;
            }
            b')' => {
                if depth <= 1 {
                    return;
                }
                depth -= 1;
                index += 1;
            }
            b'\'' | b'"' => index = string_end(source, index),
            byte if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') => {
                let Some((name, after)) = identifier_at(source, index) else {
                    index += 1;
                    continue;
                };
                if source[skip_whitespace(source, after)..].starts_with('(') {
                    names.push(short_name(name));
                }
                index = after;
            }
            _ => index += 1,
        }
    }
}

/// Returns the identifier starting at or just after `start`, and where it ends.
pub(super) fn identifier_at(source: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    let start = skip_whitespace(source, start);
    let mut end = start;
    while bytes
        .get(end)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.'))
    {
        end += 1;
    }
    (end > start).then(|| (&source[start..end], end))
}

/// Returns the last dotted segment of an annotation name.
pub(super) fn short_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// Returns the offset just past the string literal opening at `start`.
pub(super) fn string_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let quote = bytes[start];
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            byte if byte == quote => return index + 1,
            _ => index += 1,
        }
    }
    source.len()
}

/// Returns the first offset at or after `start` that is not whitespace.
pub(super) fn skip_whitespace(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut index = start;
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    index
}

/// Returns whether source imports or exports a Dust annotation package URI.
pub(super) fn contains_dust_annotation_package(source: &str) -> bool {
    source.contains("package:dust_dart/") || source.contains("package:dust_flutter/")
}

/// Returns Dust runtime package imports found in source.
pub(super) fn dust_package_imports(source: &str) -> Vec<&'static str> {
    DUST_RUNTIME_PACKAGES
        .iter()
        .filter_map(|(package, import_uri)| source.contains(import_uri).then_some(*package))
        .collect()
}
