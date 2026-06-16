use crate::{
    find_top_level_char, parse_string_literal, split_top_level_items, split_top_level_once,
};

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
    if inner.is_empty() {
        return Some(Vec::new());
    }
    split_top_level_items(inner)
        .into_iter()
        .map(parse_type_name)
        .collect()
}

/// Parses a Dart constructor invocation and returns its unprefixed name.
pub fn parse_constructor_name(source: &str) -> Option<String> {
    let source = source
        .trim()
        .strip_prefix("const ")
        .unwrap_or_else(|| source.trim())
        .trim();
    let open_paren = find_top_level_char(source, |_, ch| ch == '(')?;
    let mut name_source = source[..open_paren].trim();
    if let Some(type_args_start) = find_top_level_char(name_source, |_, ch| ch == '<') {
        name_source = name_source[..type_args_start].trim();
    }
    let full_name = parse_member_ref(name_source)?;
    full_name.rsplit('.').next().map(str::to_owned)
}

/// Parses a Dart list literal containing constructor invocations.
pub fn parse_constructor_list(source: &str) -> Option<Vec<String>> {
    let inner = source.trim().strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    Some(
        split_top_level_items(inner)
            .into_iter()
            .filter_map(parse_constructor_name)
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

#[cfg(test)]
mod tests {
    use super::{
        parse_constructor_list, parse_constructor_name, parse_member_ref, parse_string_list,
        parse_string_map, parse_type_list, parse_type_name,
    };

    #[test]
    fn parses_member_and_type_references() {
        assert_eq!(
            parse_member_ref("_private.Type"),
            Some("_private.Type".to_owned())
        );
        assert_eq!(
            parse_type_name("prefix.User"),
            Some("prefix.User".to_owned())
        );
        assert_eq!(parse_member_ref("1bad"), None);
        assert_eq!(parse_member_ref("bad-name"), None);
        assert_eq!(parse_member_ref(""), None);
    }

    #[test]
    fn parses_string_and_type_lists() {
        assert_eq!(parse_string_list("[]"), Some(Vec::new()));
        assert_eq!(
            parse_string_list("['a', \"b\"]"),
            Some(vec!["a".to_owned(), "b".to_owned()])
        );
        assert_eq!(parse_string_list("'a']"), None);
        assert_eq!(parse_string_list("['a'"), None);
        assert_eq!(parse_string_list("['a', bad]"), None);
        assert_eq!(
            parse_type_list("[User, prefix.Value]"),
            Some(vec!["User".to_owned(), "prefix.Value".to_owned()])
        );
        assert_eq!(parse_type_list("[]"), Some(Vec::new()));
        assert_eq!(parse_type_list("[User, 1bad]"), None);
        assert_eq!(parse_type_list("User]"), None);
        assert_eq!(parse_type_list("[User"), None);
        assert_eq!(parse_type_list("User"), None);
    }

    #[test]
    fn parses_constructor_names() {
        assert_eq!(
            parse_constructor_name("const prefix.CopyWith<User>()"),
            Some("CopyWith".to_owned())
        );
        assert_eq!(
            parse_constructor_name("ToString()"),
            Some("ToString".to_owned())
        );
        assert_eq!(parse_constructor_name("ToString"), None);
        assert_eq!(parse_constructor_name("1Bad()"), None);
    }

    #[test]
    fn parses_constructor_lists() {
        assert_eq!(
            parse_constructor_list("[ToString(), Eq(), const prefix.CopyWith<User>()]"),
            Some(vec![
                "ToString".to_owned(),
                "Eq".to_owned(),
                "CopyWith".to_owned(),
            ])
        );
        assert_eq!(parse_constructor_list("[]"), Some(Vec::new()));
        assert_eq!(
            parse_constructor_list("[ToString(), 1Bad()]"),
            Some(vec!["ToString".to_owned()])
        );
        assert_eq!(parse_constructor_list("ToString()]"), None);
        assert_eq!(parse_constructor_list("[ToString()"), None);
        assert_eq!(parse_constructor_list("ToString()"), None);
    }

    #[test]
    fn parses_string_maps() {
        assert_eq!(parse_string_map("{}"), Some(Vec::new()));
        assert_eq!(
            parse_string_map("{'a': 'b', \"c\": \"d\"}"),
            Some(vec![
                ("a".to_owned(), "b".to_owned()),
                ("c".to_owned(), "d".to_owned())
            ])
        );
        assert_eq!(parse_string_map("'a': 'b'}"), None);
        assert_eq!(parse_string_map("{'a': 'b'"), None);
        assert_eq!(parse_string_map("{'a' 'b'}"), None);
        assert_eq!(parse_string_map("{bad: 'b'}"), None);
        assert_eq!(parse_string_map("{'a': bad}"), None);
        assert_eq!(parse_string_map("['a']"), None);
    }
}
