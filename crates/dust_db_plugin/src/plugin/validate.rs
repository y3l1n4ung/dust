use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_plugin_api::WorkspaceAnalysis;

use super::{
    DbPluginOptions,
    analysis::{PackageDatabase, package_databases, package_row_column_map},
    model::{DbDriver, RowClass},
    parse::{database_classes, query_specs, row_classes},
};

/// Validates cached and offline SQL query metadata.
mod cache;
/// Validates annotated DAO classes and methods.
mod dao;
/// Validates parsed query specs.
mod query;
/// Validates row mapper classes and columns.
mod rows;
/// Runs SQLx describe validation against SQLite.
mod sqlx;
/// Shared DB validation type helpers.
mod types;

/// Validates DB plugin annotations and SQL query metadata for a library.
pub(crate) fn validate_db_library(
    library: &DartFileIr,
    options: DbPluginOptions,
    analysis: &WorkspaceAnalysis,
) -> Vec<Diagnostic> {
    if !options.databases && !database_classes(library).is_empty() {
        return Vec::new();
    }

    let rows = row_classes(library);
    let mut diagnostics = Vec::new();
    rows::validate_rows(&rows, &mut diagnostics);
    if options.databases {
        validate_databases(library, options, &rows, analysis, &mut diagnostics);
    }
    diagnostics
}

/// Validates database classes, DAOs, queries, and SQLx metadata.
fn validate_databases(
    library: &DartFileIr,
    options: DbPluginOptions,
    rows: &[RowClass<'_>],
    analysis: &WorkspaceAnalysis,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // These two carry a span, so they belong to the file that declares the
    // class rather than to every file that validates against it.
    for db in &database_classes(library) {
        if db.migrations.trim().is_empty() {
            diagnostics.push(Diagnostic::error(format!(
                "Database class `{}` must provide a migrations path",
                db.class.name
            )));
        }
        if matches!(db.driver, DbDriver::Postgres) {
            diagnostics.push(
                Diagnostic::error("Driver.postgres is reserved for a future Database release")
                    .with_label(dust_diagnostics::SourceLabel::new(
                        db.class.span.file_id,
                        db.class.span.range,
                        "use Driver.sqlite3 in v1",
                    )),
            );
        }
    }

    dao::validate_daos(library, rows, diagnostics);
    let queries = query_specs(library);
    for query in &queries {
        query::validate_query_shape(query, diagnostics);
    }

    let row_columns = package_row_column_map(analysis, &library.package_name);
    for query in &queries {
        query::validate_row_type_is_mapped(query, &row_columns, diagnostics);
    }

    // The schema and the row classes come from the whole package. A project
    // that keeps the database class, the row classes, and the queries in three
    // files is the normal layout, and validating each file against itself left
    // every one of those queries undescribed.
    let databases = package_databases(analysis, &library.package_name);
    let Some(db) = databases.first() else {
        return;
    };
    report_ambiguous_schema(library, &databases, diagnostics);
    sqlx::validate_sqlx_describe(library, db, &queries, &row_columns, options, diagnostics);
}

/// Reports a package that declares more than one database to validate against.
///
/// Reported once, from the library declaring the first database by name, rather
/// than once per library in the package.
fn report_ambiguous_schema(
    library: &DartFileIr,
    databases: &[PackageDatabase],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if databases.len() < 2 {
        return;
    }
    let local = database_classes(library);
    if !local.iter().any(|db| db.class.name == databases[0].name) {
        return;
    }
    let names = databases
        .iter()
        .map(|db| format!("`{}`", db.name))
        .collect::<Vec<_>>()
        .join(", ");
    diagnostics.push(Diagnostic::error(format!(
        "Package `{}` declares more than one SqlxDatabase ({names}), so a query has no one schema \
         to be validated against. Keep one database per package.",
        library.package_name
    )));
}

#[cfg(test)]
#[path = "validate/tests.rs"]
/// Unit tests for DB validation helpers.
mod tests;
