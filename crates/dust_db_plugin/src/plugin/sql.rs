/// Result of rewriting SQLx-style placeholders into SQLite placeholders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlaceholderRewrite {
    /// SQL text with `$n` placeholders rewritten to `?`.
    pub(crate) sql: String,
    /// One-based user parameter index for each expanded SQLite placeholder.
    pub(crate) parameter_order: Vec<usize>,
}

impl PlaceholderRewrite {
    /// Returns the number of placeholders after repeated parameters are expanded.
    pub(crate) fn expanded_parameter_count(&self) -> usize {
        self.parameter_order.len()
    }
}

/// Rewrites SQLx-style `$1` placeholders into SQLite `?` placeholders.
///
/// A `$n` only means a parameter where the database would bind one. Everything
/// that can hold the same three characters without meaning that is copied
/// through byte for byte: string literals, quoted identifiers, `--` line
/// comments, `/* */` block comments, and Postgres dollar-quoted bodies.
///
/// Block comments do not nest, which is what SQLite does: the first `*/` ends
/// the comment however many `/*` preceded it. An unterminated literal or
/// comment runs to the end of the statement rather than erroring here, leaving
/// the database to reject it with a message about the SQL the user wrote.
pub(crate) fn rewrite_sqlite_placeholders(
    sql: &str,
    user_parameter_count: usize,
) -> Result<PlaceholderRewrite, String> {
    let mut rewritten = String::new();
    let mut order = Vec::<usize>::new();
    let mut i = 0;

    while i < sql.len() {
        if let Some(end) = verbatim_span(sql, i) {
            rewritten.push_str(&sql[i..end]);
            i = end;
            continue;
        }
        if let Some((index, end)) = placeholder_at(sql, i)? {
            order.push(index);
            rewritten.push('?');
            i = end;
            continue;
        }
        let ch = sql[i..].chars().next().unwrap_or_default();
        rewritten.push(ch);
        i += ch.len_utf8();
    }

    let max = order.iter().copied().max().unwrap_or(0);
    for index in 1..=max {
        if !order.contains(&index) {
            return Err(format!("SQL placeholders must not skip `${index}`"));
        }
    }
    if user_parameter_count != max {
        return Err(format!(
            "query binds {user_parameter_count} args but SQL expects {max} parameters"
        ));
    }

    Ok(PlaceholderRewrite {
        sql: rewritten,
        parameter_order: order,
    })
}

/// Returns the end of the construct starting at `start` that must be copied unchanged.
fn verbatim_span(sql: &str, start: usize) -> Option<usize> {
    let rest = &sql[start..];
    if rest.starts_with('\'') {
        return Some(quoted_end(sql, start, '\''));
    }
    if rest.starts_with('"') {
        return Some(quoted_end(sql, start, '"'));
    }
    if rest.starts_with("--") {
        return Some(sql[start..].find('\n').map_or(sql.len(), |at| start + at));
    }
    if rest.starts_with("/*") {
        let body = start + "/*".len();
        return Some(
            sql[body..]
                .find("*/")
                .map_or(sql.len(), |at| body + at + "*/".len()),
        );
    }
    dollar_quote_end(sql, start)
}

/// Returns the end of a quoted run, where a doubled quote escapes rather than closes.
fn quoted_end(sql: &str, start: usize, quote: char) -> usize {
    let mut i = start + quote.len_utf8();
    while let Some(at) = sql[i..].find(quote) {
        let after = i + at + quote.len_utf8();
        if sql[after..].starts_with(quote) {
            i = after + quote.len_utf8();
            continue;
        }
        return after;
    }
    sql.len()
}

/// Returns the end of a Postgres dollar-quoted body, `$$...$$` or `$tag$...$tag$`.
fn dollar_quote_end(sql: &str, start: usize) -> Option<usize> {
    let rest = sql[start..].strip_prefix('$')?;
    let tag_len = rest
        .find(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
        .unwrap_or(rest.len());
    let tag = &rest[..tag_len];
    // `$1` opens no literal. A tag may not begin with a digit, which is exactly
    // what keeps a placeholder from being read as one.
    if tag.starts_with(|ch: char| ch.is_ascii_digit()) {
        return None;
    }
    if !rest[tag_len..].starts_with('$') {
        return None;
    }
    let delimiter = format!("${tag}$");
    let body = start + delimiter.len();
    // A dollar quote the database would reject — no closing tag — is not one
    // here either. Treating it as one would swallow the rest of the statement.
    let end = sql[body..].find(&delimiter)?;
    Some(body + end + delimiter.len())
}

/// Reads a `$n` placeholder at `start`, returning its index and end offset.
fn placeholder_at(sql: &str, start: usize) -> Result<Option<(usize, usize)>, String> {
    let Some(rest) = sql[start..].strip_prefix('$') else {
        return Ok(None);
    };
    let digits = rest
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(rest.len());
    if digits == 0 {
        return Ok(None);
    }
    let end = start + '$'.len_utf8() + digits;
    let index = rest[..digits]
        .parse::<usize>()
        .map_err(|_| "invalid SQL placeholder".to_owned())?;
    if index == 0 {
        return Err("SQL placeholders are 1-based".to_owned());
    }
    Ok(Some((index, end)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_sqlite_placeholders_and_preserves_order() {
        let rewrite = rewrite_sqlite_placeholders(
            r"SELECT '$1', id FROM users WHERE id = $1 OR owner_id = $1 AND name = $2",
            2,
        )
        .unwrap();

        assert_eq!(
            rewrite.sql,
            r"SELECT '$1', id FROM users WHERE id = ? OR owner_id = ? AND name = ?"
        );
        assert_eq!(rewrite.parameter_order, vec![1, 1, 2]);
        assert_eq!(rewrite.expanded_parameter_count(), 3);
    }

    #[test]
    fn supports_reordered_placeholders() {
        let rewrite =
            rewrite_sqlite_placeholders(r"SELECT * FROM users WHERE org_id = $2 AND id = $1", 2)
                .unwrap();

        assert_eq!(
            rewrite.sql,
            r"SELECT * FROM users WHERE org_id = ? AND id = ?"
        );
        assert_eq!(rewrite.parameter_order, vec![2, 1]);
    }

    #[test]
    fn ignores_placeholders_in_line_comments() {
        let rewrite = rewrite_sqlite_placeholders(
            "-- filter by $9 owner\nSELECT count(*) FROM orders WHERE account_id = $1",
            1,
        )
        .unwrap();

        assert_eq!(
            rewrite.sql,
            "-- filter by $9 owner\nSELECT count(*) FROM orders WHERE account_id = ?"
        );
        assert_eq!(rewrite.parameter_order, vec![1]);
    }

    #[test]
    fn ignores_placeholders_in_block_comments() {
        let rewrite = rewrite_sqlite_placeholders(
            "DELETE FROM orders WHERE id = $1 /* owner is $2 */ AND account_id = $2",
            2,
        )
        .unwrap();

        assert_eq!(
            rewrite.sql,
            "DELETE FROM orders WHERE id = ? /* owner is $2 */ AND account_id = ?"
        );
        assert_eq!(rewrite.parameter_order, vec![1, 2]);
    }

    #[test]
    fn block_comments_do_not_nest() {
        // SQLite ends the comment at the first `*/`, so `$1` after it is a real
        // parameter however many `/*` came before.
        let rewrite =
            rewrite_sqlite_placeholders("SELECT /* a /* b $9 */ id FROM t WHERE id = $1", 1)
                .unwrap();

        assert_eq!(rewrite.sql, "SELECT /* a /* b $9 */ id FROM t WHERE id = ?");
        assert_eq!(rewrite.parameter_order, vec![1]);
    }

    #[test]
    fn unterminated_block_comment_runs_to_the_end() {
        let rewrite = rewrite_sqlite_placeholders("SELECT id FROM t /* $1", 0).unwrap();

        assert_eq!(rewrite.sql, "SELECT id FROM t /* $1");
        assert!(rewrite.parameter_order.is_empty());
    }

    #[test]
    fn ignores_placeholders_in_dollar_quoted_bodies() {
        let rewrite = rewrite_sqlite_placeholders(
            "SELECT $$ raw $1 body $$, $tag$ also $2 $tag$ FROM t WHERE id = $1",
            1,
        )
        .unwrap();

        assert_eq!(
            rewrite.sql,
            "SELECT $$ raw $1 body $$, $tag$ also $2 $tag$ FROM t WHERE id = ?"
        );
        assert_eq!(rewrite.parameter_order, vec![1]);
    }

    #[test]
    fn an_unclosed_dollar_tag_is_not_a_literal() {
        let rewrite = rewrite_sqlite_placeholders("SELECT $tag$ id FROM t WHERE id = $1", 1)
            .expect("a lone `$tag$` must not swallow the statement");

        assert_eq!(rewrite.sql, "SELECT $tag$ id FROM t WHERE id = ?");
        assert_eq!(rewrite.parameter_order, vec![1]);
    }

    #[test]
    fn comment_openers_inside_string_literals_are_text() {
        let rewrite = rewrite_sqlite_placeholders(
            r"SELECT '-- not a comment', '/*', id FROM t WHERE id = $1",
            1,
        )
        .unwrap();

        assert_eq!(
            rewrite.sql,
            r"SELECT '-- not a comment', '/*', id FROM t WHERE id = ?"
        );
        assert_eq!(rewrite.parameter_order, vec![1]);
    }

    #[test]
    fn quote_openers_inside_comments_are_text() {
        let rewrite =
            rewrite_sqlite_placeholders("-- it's $9 here\nSELECT id FROM t WHERE id = $1", 1)
                .unwrap();

        assert_eq!(
            rewrite.sql,
            "-- it's $9 here\nSELECT id FROM t WHERE id = ?"
        );
        assert_eq!(rewrite.parameter_order, vec![1]);
    }

    #[test]
    fn doubled_quotes_escape_rather_than_close() {
        let rewrite =
            rewrite_sqlite_placeholders(r#"SELECT 'it''s $9', "od""d $9" FROM t WHERE id = $1"#, 1)
                .unwrap();

        assert_eq!(
            rewrite.sql,
            r#"SELECT 'it''s $9', "od""d $9" FROM t WHERE id = ?"#
        );
        assert_eq!(rewrite.parameter_order, vec![1]);
    }

    #[test]
    fn rejects_skipped_and_zero_placeholders() {
        assert_eq!(
            rewrite_sqlite_placeholders("SELECT id FROM t WHERE id = $2", 2),
            Err("SQL placeholders must not skip `$1`".to_owned())
        );
        assert_eq!(
            rewrite_sqlite_placeholders("SELECT id FROM t WHERE id = $0", 1),
            Err("SQL placeholders are 1-based".to_owned())
        );
    }
}
