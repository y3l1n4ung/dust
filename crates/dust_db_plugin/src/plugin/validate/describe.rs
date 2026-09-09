//! Describing queries against a live database, and checking what comes back.
//!
//! Split from `sqlx.rs`, which owns connecting and migrating. This half owns
//! what the server says about a query once it is reachable: the describe call
//! itself, and whether the columns it reports match the Dart row that reads
//! them.

use std::collections::{HashMap, HashSet};

use either::Either;
use sqlx::{Column, Connection, Executor};

use crate::plugin::{
    analysis::RowColumn,
    column_alias::{NullabilityOverride, parse_column_alias},
    column_types::accepts,
    dialect::Dialect,
    model::{QueryFunction, QuerySpec},
};

use super::{
    cache::QueryCacheEntry,
    hash::stable_hash_hex,
    query::{query_row_type, validate_placeholders},
    sqlx::DescribeRequest,
};

/// Describes all queries and returns metadata suitable for cache writes.
pub(super) async fn describe_queries<C>(
    conn: &mut C,
    request: &DescribeRequest<'_>,
    warnings: &mut Vec<String>,
) -> Result<Vec<QueryCacheEntry>, String>
where
    C: Connection,
    for<'e> &'e mut C: Executor<'e, Database = C::Database>,
{
    let DescribeRequest {
        dialect,
        migrations,
        schema_hash,
        queries,
        row_columns,
        typed_columns,
    } = *request;
    let mut metadata = Vec::new();
    for query in queries {
        // Unchecked SQL is not described and never enters the cache: its text
        // may be dynamic, and pretending otherwise would put an entry in the
        // committed cache that no build can reproduce.
        if !query.sql_source_static || matches!(query.function, QueryFunction::Unsafe) {
            continue;
        }
        // Arity and gap checking is dialect-independent, so it always runs. Which
        // text the database is asked about is not: SQLite receives `?` and
        // PostgreSQL receives the `$n` as written.
        let rewrite = validate_placeholders(&query.sql, query.parameter_count)?;
        let (described_sql, expected_parameters) = if dialect.rewrites_placeholders {
            (rewrite.sql.as_str(), rewrite.expanded_parameter_count())
        } else {
            (query.sql.as_str(), query.parameter_count)
        };
        let describe = conn
            .describe(described_sql)
            .await
            .map_err(|error| format!("SQLx rejected `{}`: {error}", query.display_name()))?;
        let parameter_count = describe.parameters().map_or(0, |params| match params {
            Either::Left(values) => values.len(),
            Either::Right(count) => count,
        });
        if parameter_count != expected_parameters {
            // The bind list is built from the placeholders Dust replaced, so a
            // disagreement here means the generated call would bind the wrong
            // number of arguments. Show the SQL the database actually parsed:
            // the `$n` that is not where it looks is visible in it.
            return Err(format!(
                "SQLx query `{}` would bind {} arguments to a statement the database reads as \
                 having {parameter_count} parameters. A `$n` was rewritten somewhere it cannot \
                 be bound:\n{}",
                query.display_name(),
                expected_parameters,
                described_sql.trim()
            ));
        }
        validate_described_columns(query, row_columns, &describe)?;
        warnings.extend(describe_column_warnings(
            query,
            dialect,
            typed_columns,
            &describe,
        ));
        metadata.push(QueryCacheEntry {
            driver: dialect.name.to_owned(),
            migrations: migrations.to_owned(),
            schema_hash: schema_hash.to_owned(),
            sql_hash: stable_hash_hex(query.sql.as_bytes()),
            sql: query.sql.clone(),
            user_parameter_count: query.parameter_count,
            expanded_parameter_count: expected_parameters,
            fetch_mode: query.fetch.as_str().to_owned(),
            row_type: query.row_type.clone(),
            columns: describe
                .columns()
                .iter()
                .map(|column| parse_column_alias(column.name()).name)
                .collect(),
        });
    }
    Ok(metadata)
}

/// Validates SQLx-described columns against scalar and row requirements.
fn validate_described_columns<DB: sqlx::Database>(
    query: &QuerySpec,
    row_columns: &HashMap<String, HashSet<String>>,
    describe: &sqlx::Describe<DB>,
) -> Result<(), String> {
    if matches!(query.function, QueryFunction::Scalar) && describe.columns().len() != 1 {
        return Err(format!(
            "SQLx query `{}` must return exactly one scalar column",
            query.display_name()
        ));
    }
    let Some(row_type) = query_row_type(query) else {
        return Ok(());
    };
    let Some(required_columns) = row_columns.get(row_type) else {
        return Ok(());
    };
    // A `foo!` or `foo?` alias is a nullability override, and the marker is not
    // part of the name a row class spells.
    let returned_columns = describe
        .columns()
        .iter()
        .map(|column| parse_column_alias(column.name()).name)
        .collect::<HashSet<_>>();
    if let Some(missing) = required_columns
        .iter()
        .find(|column| !returned_columns.contains(*column))
    {
        return Err(format!(
            "SQLx query `{}` does not return required column `{missing}` for row `{row_type}`",
            query.display_name()
        ));
    }
    Ok(())
}

/// Reports described types and nullability that disagree with the row class.
///
/// Warnings rather than errors: the accepted type table is deliberately
/// permissive, and a dialect whose inference is wrong about nullability has the
/// column-alias overrides as its answer. A finding here is worth reading, not
/// worth failing a build over yet.
fn describe_column_warnings<DB: sqlx::Database>(
    query: &QuerySpec,
    dialect: &Dialect,
    typed_columns: &HashMap<String, Vec<RowColumn>>,
    describe: &sqlx::Describe<DB>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    let Some(row_type) = query_row_type(query) else {
        return warnings;
    };
    let Some(fields) = typed_columns.get(row_type) else {
        return warnings;
    };

    for (index, column) in describe.columns().iter().enumerate() {
        let alias = parse_column_alias(column.name());
        let Some(field) = fields.iter().find(|field| field.name == alias.name) else {
            continue;
        };

        let sql_type = column.type_info().to_string();
        if !accepts(dialect.driver, &field.dart_type, &sql_type) {
            warnings.push(format!(
                "SQLx query `{}` reads column `{}` of type `{sql_type}` into `{}`, which cannot hold it",
                query.display_name(),
                alias.name,
                field.dart_type,
            ));
        }

        // An override says what the database could not know, so it settles the
        // question. Inference is only consulted for a dialect whose inference
        // is worth consulting.
        let described_nullable = match alias.nullability {
            NullabilityOverride::NotNull => Some(false),
            NullabilityOverride::Nullable => Some(true),
            NullabilityOverride::Inferred if dialect.checks_nullability => describe.nullable(index),
            NullabilityOverride::Inferred => None,
        };
        if described_nullable == Some(true) && !field.nullable {
            warnings.push(format!(
                "SQLx query `{}` reads nullable column `{}` into non-nullable `{}`. Make the field nullable, or write `as \"{}!\"` if the database is wrong about it",
                query.display_name(),
                alias.name,
                field.dart_type,
                alias.name,
            ));
        }
    }
    warnings
}
