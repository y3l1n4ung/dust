/// Normalizes assembled generated source into a stable final text form.
pub(crate) fn format_generated_source(mut source: String) -> String {
    while source.ends_with('\n') {
        source.pop();
    }
    source.push('\n');
    source
}

#[cfg(test)]
mod tests {
    use super::format_generated_source;

    #[test]
    fn normalizes_trailing_newlines_idempotently() {
        let source = "mixin _$User {}\n\n\n".to_owned();

        let formatted = format_generated_source(source);
        let formatted_again = format_generated_source(formatted.clone());

        assert_eq!(formatted, "mixin _$User {}\n");
        assert_eq!(formatted_again, formatted);
    }

    #[test]
    fn preserves_grammar_blocked_future_syntax_verbatim() {
        let source =
            "enum ProductStatus { ready, hidden }\n\nfinal status = .ready;\n\n".to_owned();

        let formatted = format_generated_source(source);

        assert_eq!(
            formatted,
            "enum ProductStatus { ready, hidden }\n\nfinal status = .ready;\n"
        );
    }
}
