use dust_text::{FileId, SourceText};

use super::{ParsedQueryFunction, extract_query_calls};

#[test]
fn extracts_query_calls() {
    let source = SourceText::new(
        FileId::new(1),
        "queryAs<UserRow>(r'SELECT * FROM users WHERE id = $1', [id]).fetchOptional(db);",
    );

    let calls = extract_query_calls(&source);

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].function, ParsedQueryFunction::As);
    assert_eq!(calls[0].type_arg_source.as_deref(), Some("UserRow"));
    assert_eq!(calls[0].sql, "SELECT * FROM users WHERE id = $1");
    assert!(calls[0].sql_source_static);
    assert_eq!(calls[0].parameter_count, 1);
    assert!(calls[0].params_source_is_list);
    assert_eq!(calls[0].fetch_method.as_deref(), Some("fetchOptional"));
}

#[test]
fn rejects_dynamic_sql_and_non_list_params() {
    let source = SourceText::new(
        FileId::new(1),
        "queryRaw('SELECT * FROM $table', args).fetch(db);",
    );

    let calls = extract_query_calls(&source);

    assert_eq!(calls.len(), 1);
    assert!(!calls[0].sql_source_static);
    assert!(!calls[0].params_source_is_list);
}

#[test]
fn ignores_strings_comments_and_prefixes() {
    let source = SourceText::new(
        FileId::new(1),
        "final text = 'queryRaw(r\"SELECT 1\", [])'; // queryRaw(r'SELECT 1', [])\nmyqueryRaw(r'SELECT 1', []);",
    );

    let calls = extract_query_calls(&source);

    assert_eq!(calls.len(), 0);
}

#[test]
fn extracts_multiline_raw_sql() {
    let source = SourceText::new(
        FileId::new(1),
        "queryRaw(r'''\nSELECT *\nFROM users\nWHERE id = $1\n''', [id]).fetch(db);",
    );

    let calls = extract_query_calls(&source);

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].function, ParsedQueryFunction::Raw);
    assert_eq!(calls[0].sql, "\nSELECT *\nFROM users\nWHERE id = $1\n");
    assert_eq!(calls[0].parameter_count, 1);
    assert_eq!(calls[0].fetch_method.as_deref(), Some("fetch"));
}

#[test]
fn rejects_concatenated_and_variable_sql() {
    let source = SourceText::new(
        FileId::new(1),
        "queryRaw('SELECT * ' 'FROM users', []).fetch(db); queryRaw(sql, []).fetch(db);",
    );

    let calls = extract_query_calls(&source);

    assert_eq!(calls.len(), 2);
    assert!(!calls[0].sql_source_static);
    assert!(!calls[1].sql_source_static);
}

#[test]
fn extracts_fetch_modes_and_default_params() {
    let source = SourceText::new(
        FileId::new(1),
        r#"
queryScalar<int>(r'SELECT COUNT(*) FROM users', const <Object?>[]).fetchOptional(db);
queryRaw(r'SELECT * FROM users').fetch(db);
queryExecute(r'DELETE FROM users').execute(db);
queryAs<List<UserRow>>(r'SELECT * FROM users', [orgId]).fetchAll(db);
"#,
    );

    let calls = extract_query_calls(&source);

    assert_eq!(calls.len(), 4);
    assert_eq!(calls[0].function, ParsedQueryFunction::Scalar);
    assert_eq!(calls[0].fetch_method.as_deref(), Some("fetchOptional"));
    assert_eq!(calls[0].type_arg_source.as_deref(), Some("int"));
    assert_eq!(calls[0].parameter_count, 0);
    assert_eq!(calls[1].function, ParsedQueryFunction::Raw);
    assert_eq!(calls[1].fetch_method.as_deref(), Some("fetch"));
    assert_eq!(calls[1].parameter_count, 0);
    assert_eq!(calls[2].function, ParsedQueryFunction::Execute);
    assert_eq!(calls[2].fetch_method.as_deref(), Some("execute"));
    assert_eq!(calls[3].function, ParsedQueryFunction::As);
    assert_eq!(calls[3].fetch_method.as_deref(), Some("fetchAll"));
    assert_eq!(calls[3].type_arg_source.as_deref(), Some("List<UserRow>"));
}

#[test]
fn ignores_unbalanced_query_calls() {
    let source = SourceText::new(
        FileId::new(1),
        "queryRaw <Row r'SELECT 1'; queryRaw(r'SELECT 1', []",
    );

    let calls = extract_query_calls(&source);

    assert_eq!(calls.len(), 0);
}
