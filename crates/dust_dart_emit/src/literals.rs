/// Renders a Dart single-quoted string literal.
pub fn dart_string_literal(value: &str) -> String {
    format!("'{}'", escape_dart_string(value))
}

/// Escapes content for a single-quoted Dart string literal.
pub fn escape_dart_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('$', "\\$")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::{dart_string_literal, escape_dart_string};

    #[test]
    fn escapes_single_quoted_dart_strings() {
        assert_eq!(
            dart_string_literal("can't pay $1\n"),
            r#"'can\'t pay \$1\n'"#
        );
    }

    #[test]
    fn escapes_literal_content_without_quotes() {
        assert_eq!(escape_dart_string("a\\b\tc"), r#"a\\b\tc"#);
    }
}
