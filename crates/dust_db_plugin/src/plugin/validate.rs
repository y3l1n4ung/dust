use dust_diagnostics::Diagnostic;
use dust_ir::LibraryIr;

use super::{
    DbPluginOptions,
    model::{DbDriver, RowClass},
    parse::{database_classes, query_specs, row_classes},
};

mod cache;
mod dao;
mod query;
mod rows;
mod sqlx;
mod types;

pub(crate) fn validate_db_library(
    library: &LibraryIr,
    options: DbPluginOptions,
) -> Vec<Diagnostic> {
    if !options.databases && !database_classes(library).is_empty() {
        return Vec::new();
    }

    let rows = row_classes(library);
    let mut diagnostics = Vec::new();
    rows::validate_rows(&rows, &mut diagnostics);
    if options.databases {
        validate_databases(library, options, &rows, &mut diagnostics);
    }
    diagnostics
}

fn validate_databases(
    library: &LibraryIr,
    options: DbPluginOptions,
    rows: &[RowClass<'_>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let databases = database_classes(library);
    if databases.is_empty() {
        return;
    }
    let row_columns = rows::row_column_map(rows);
    let queries = query_specs(library);
    for db in &databases {
        if db.migrations.trim().is_empty() {
            diagnostics.push(Diagnostic::error(format!(
                "Database class `{}` must provide a migrations path",
                db.class.name
            )));
        }
        if matches!(db.driver, DbDriver::Postgres) {
            diagnostics.push(
                Diagnostic::error("Driver.postgres is reserved for a future Dust DB release")
                    .with_label(dust_diagnostics::SourceLabel::new(
                        db.class.span.file_id,
                        db.class.span.range,
                        "use Driver.sqlite3 in v1",
                    )),
            );
        }
    }
    dao::validate_daos(library, rows, diagnostics);
    for query in &queries {
        query::validate_query_shape(query, diagnostics);
    }
    if let Some(db) = databases.first() {
        sqlx::validate_sqlx_describe(library, db, &queries, &row_columns, options, diagnostics);
    }
}

#[cfg(test)]
#[path = "validate/tests.rs"]
mod tests;
