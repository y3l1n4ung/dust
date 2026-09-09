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
#[path = "sql/tests.rs"]
mod tests;
