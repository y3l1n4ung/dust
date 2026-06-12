use crate::{parse_string_literal, split_top_level_items, split_top_level_once};

/// Parses a Dart member or type reference source.
pub fn parse_member_ref(source: &str) -> Option<String> {
    let source = source.trim();
    let mut chars = source.chars();
    let first = chars.next()?;
    (first == '_' || first.is_ascii_alphabetic())
        .then_some(())
        .and_then(|_| {
            chars
                .all(|ch| ch == '_' || ch == '.' || ch.is_ascii_alphanumeric())
                .then(|| source.to_owned())
        })
}

/// Parses a Dart type name source.
pub fn parse_type_name(source: &str) -> Option<String> {
    parse_member_ref(source)
}

/// Parses a Dart list literal containing string literals.
pub fn parse_string_list(source: &str) -> Option<Vec<String>> {
    let inner = source.trim().strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    split_top_level_items(inner)
        .into_iter()
        .map(parse_string_literal)
        .collect()
}

/// Parses a Dart list literal containing type/member references.
pub fn parse_type_list(source: &str) -> Option<Vec<String>> {
    let inner = source.trim().strip_prefix('[')?.strip_suffix(']')?.trim();
    Some(
        split_top_level_items(inner)
            .into_iter()
            .filter_map(parse_type_name)
            .collect(),
    )
}

/// Parses a Dart map literal with string literal keys and values.
pub fn parse_string_map(source: &str) -> Option<Vec<(String, String)>> {
    let inner = source.trim().strip_prefix('{')?.strip_suffix('}')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    split_top_level_items(inner)
        .into_iter()
        .map(|item| {
            let (key, value) = split_top_level_once(item, ':')?;
            Some((
                parse_string_literal(key.trim())?,
                parse_string_literal(value.trim())?,
            ))
        })
        .collect()
}
