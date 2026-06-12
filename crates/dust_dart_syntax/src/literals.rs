/// Parses a single or double quoted Dart string literal without unescaping.
pub fn parse_string_literal(source: &str) -> Option<String> {
    let source = source.trim();
    let source = source
        .strip_prefix('r')
        .or_else(|| source.strip_prefix('R'))
        .unwrap_or(source);
    let first = source.chars().next()?;
    let last = source.chars().next_back()?;
    if source.len() < 2 || first != last || !matches!(first, '\'' | '"') {
        return None;
    }
    Some(source[1..source.len() - 1].to_owned())
}

/// Parses a Dart boolean literal.
pub fn parse_bool_literal(source: &str) -> Option<bool> {
    match source.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Parses a static Dart string literal and rejects interpolation.
///
/// Raw strings (`r'...'`) and triple-quoted strings are supported. Adjacent
/// string literal concatenation is intentionally rejected so checked SQL can be
/// validated from one exact source literal.
pub fn parse_static_dart_string_literal(source: &str) -> Option<String> {
    let source = source.trim();
    let (raw, source) = match source.as_bytes() {
        [b'r' | b'R', b'\'' | b'"', ..] => (true, &source[1..]),
        _ => (false, source),
    };
    let quote = source.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }
    let delimiter = if source.starts_with(&quote.to_string().repeat(3)) {
        quote.to_string().repeat(3)
    } else {
        quote.to_string()
    };
    let body_start = delimiter.len();

    let mut value = String::new();
    let mut escaped = false;
    let mut end_offset = None;
    for (index, ch) in source[body_start..].char_indices() {
        let absolute = body_start + index;
        if !raw && escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        if !raw && ch == '\\' {
            escaped = true;
            continue;
        }
        if !raw && ch == '$' {
            return None;
        }
        if source[absolute..].starts_with(&delimiter) {
            end_offset = Some(absolute + delimiter.len());
            break;
        }
        value.push(ch);
    }

    let end_offset = end_offset?;
    source[end_offset..].trim().is_empty().then_some(value)
}

#[cfg(test)]
mod tests {
    use super::parse_static_dart_string_literal;

    #[test]
    fn parses_static_dart_strings() {
        assert_eq!(
            parse_static_dart_string_literal(r"r'''SELECT * FROM users WHERE id = $1'''"),
            Some("SELECT * FROM users WHERE id = $1".to_owned())
        );
        assert_eq!(
            parse_static_dart_string_literal("'SELECT * FROM users'"),
            Some("SELECT * FROM users".to_owned())
        );
        assert_eq!(
            parse_static_dart_string_literal("'SELECT * FROM $table'"),
            None
        );
        assert_eq!(
            parse_static_dart_string_literal("'SELECT * ' 'FROM users'"),
            None
        );
    }
}
