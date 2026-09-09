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
        rewrite_sqlite_placeholders("SELECT /* a /* b $9 */ id FROM t WHERE id = $1", 1).unwrap();

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
        rewrite_sqlite_placeholders("-- it's $9 here\nSELECT id FROM t WHERE id = $1", 1).unwrap();

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
