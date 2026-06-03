#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlaceholderRewrite {
    pub(crate) sql: String,
    pub(crate) parameter_order: Vec<usize>,
}

impl PlaceholderRewrite {
    pub(crate) fn expanded_parameter_count(&self) -> usize {
        self.parameter_order.len()
    }
}

pub(crate) fn rewrite_sqlite_placeholders(
    sql: &str,
    user_parameter_count: usize,
) -> Result<PlaceholderRewrite, String> {
    let mut rewritten = String::new();
    let mut order = Vec::<usize>::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let bytes = sql.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let ch = sql[i..].chars().next().unwrap_or_default();
        if ch == '\'' && !in_double_quote {
            rewritten.push(ch);
            if in_single_quote && sql[i + ch.len_utf8()..].starts_with('\'') {
                i += ch.len_utf8();
                rewritten.push('\'');
            } else {
                in_single_quote = !in_single_quote;
            }
            i += ch.len_utf8();
            continue;
        }
        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            rewritten.push(ch);
            i += ch.len_utf8();
            continue;
        }
        if ch == '$' && !in_single_quote && !in_double_quote {
            let mut end = i + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > i + 1 {
                let index = sql[i + 1..end]
                    .parse::<usize>()
                    .map_err(|_| "invalid SQL placeholder".to_owned())?;
                if index == 0 {
                    return Err("SQL placeholders are 1-based".to_owned());
                }
                order.push(index);
                rewritten.push('?');
                i = end;
                continue;
            }
        }
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
}
