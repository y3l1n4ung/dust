use dust_dart_emit::{DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_INT, DART_NUM, DART_STRING};
use dust_ir::TypeIr;

/// Lowercases the first ASCII character of a generated identifier.
pub(super) fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_ascii_lowercase(),
        chars.collect::<String>()
    )
}

/// Escapes text for a generated Dart single-quoted string literal.
pub(super) fn escape_dart_string(source: &str) -> String {
    source
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('$', "\\$")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Renders SQL as a Dart string literal.
///
/// `r'''...'''` is preferred: it keeps SQL readable in generated output and
/// needs no escaping of the backslashes and `$n` placeholders SQL is full of.
/// It is not always safe, and the unsafe cases were measured against the Dart
/// analyzer rather than reasoned about from its grammar.
///
/// | SQL | raw form | why |
/// | :--- | :--- | :--- |
/// | holds `'''` | rejected | closes the literal early |
/// | ends on `'` | rejected | merges with the closing delimiter |
/// | ends on `''` | **wrong value** | the delimiter eats one quote, silently |
/// | holds `\r` | **wrong value** | the source reader drops it, silently |
///
/// The last two are the dangerous ones. They compile, so the generated query
/// runs against the database carrying SQL the caller never wrote. Both take the
/// escaped form instead, which spells every byte out and round-trips exactly.
///
/// A quote anywhere but the end stays raw, since it cannot reach the delimiter,
/// and so do `$`, backslashes, tabs and newlines, which a raw string carries
/// through untouched.
pub(super) fn render_sql_literal(source: &str) -> String {
    let closes_early = source.contains("'''");
    let merges_with_delimiter = source.ends_with('\'');
    let loses_a_carriage_return = source.contains('\r');

    if !closes_early && !merges_with_delimiter && !loses_a_carriage_return {
        return format!("r'''{source}'''");
    }
    format!("'{}'", escape_dart_string(source))
}

/// Returns true when a type can be queried through scalar helpers.
pub(super) fn is_scalar_type(ty: &TypeIr) -> bool {
    matches!(
        ty.name(),
        Some(DART_STRING | DART_INT | DART_DOUBLE | DART_NUM | DART_BOOL | DART_DATE_TIME)
    )
}

#[cfg(test)]
mod tests {
    use super::render_sql_literal;

    /// Every shape of SQL that reaches the emitter, and the form it must take.
    ///
    /// The expectations are what the Dart analyzer accepts and what `dart run`
    /// reads back byte for byte. The table was built by putting each case
    /// through Dart rather than by reading its grammar.
    #[test]
    fn each_shape_of_sql_renders_to_a_literal_dart_reads_back() {
        let cases: &[(&str, &str, &str)] = &[
            // Safe raw, which is the readable form.
            ("plain SQL", "SELECT 1", "r'''SELECT 1'''"),
            (
                "a quote in the middle cannot reach the delimiter",
                "WHERE s = 'x' AND n > 0",
                "r'''WHERE s = 'x' AND n > 0'''",
            ),
            ("a leading quote is fine", "'x' = s", "r''''x' = s'''"),
            (
                "placeholders stay literal, which is why raw is preferred",
                "WHERE id = $1",
                "r'''WHERE id = $1'''",
            ),
            (
                "backslashes stay literal too",
                "WHERE p LIKE 'a\\_b' ESCAPE x",
                "r'''WHERE p LIKE 'a\\_b' ESCAPE x'''",
            ),
            ("tabs survive", "SELECT\t1", "r'''SELECT\t1'''"),
            (
                "newlines survive",
                "SELECT 1\nFROM t",
                "r'''SELECT 1\nFROM t'''",
            ),
            ("empty SQL", "", "r''''''"),
            // Dart rejects these outright: the generated file does not compile.
            (
                "ending on a quote merges with the delimiter",
                "WHERE s = 'x'",
                "'WHERE s = \\'x\\''",
            ),
            (
                "the delimiter itself closes the literal early",
                "SELECT '''",
                "'SELECT \\'\\'\\''",
            ),
            // Dart accepts these and reads back the wrong value. Worse than a
            // compile error: the query runs, carrying SQL nobody wrote.
            (
                "ending on two quotes silently loses one",
                "WHERE s = '' AND t = ''",
                "'WHERE s = \\'\\' AND t = \\'\\''",
            ),
            (
                "a carriage return is silently dropped",
                "SELECT 1\r\nFROM t",
                "'SELECT 1\\r\\nFROM t'",
            ),
        ];

        for (why, sql, expected) in cases {
            assert_eq!(&render_sql_literal(sql).as_str(), expected, "{why}");
        }
    }

    /// The reported case, named so the report stays findable from the code.
    #[test]
    fn issue_537_a_query_filtering_on_a_string_constant() {
        let rendered = render_sql_literal("SELECT total FROM products WHERE status = 'published'");

        assert!(!rendered.contains("''''"), "{rendered}");
        assert_eq!(
            rendered,
            "'SELECT total FROM products WHERE status = \\'published\\''"
        );
    }
}
