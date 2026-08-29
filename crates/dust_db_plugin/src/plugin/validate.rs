use std::collections::{HashMap, HashSet};

use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_plugin_api::WorkspaceAnalysis;

use super::{
    DbPluginOptions,
    analysis::{PackageDatabase, duplicate_row_types, package_databases, package_row_column_map},
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
    let package_rows = package_row_column_map(analysis, &library.package_name);
    let mut diagnostics = Vec::new();
    rows::validate_rows(&rows, &package_rows, &mut diagnostics);
    if options.databases {
        validate_databases(
            library,
            options,
            &rows,
            analysis,
            &package_rows,
            &mut diagnostics,
        );
    }
    diagnostics
}

/// Validates database classes, DAOs, queries, and SQLx metadata.
fn validate_databases(
    library: &DartFileIr,
    options: DbPluginOptions,
    rows: &[RowClass<'_>],
    analysis: &WorkspaceAnalysis,
    row_columns: &HashMap<String, HashSet<String>>,
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

    let ambiguous = report_ambiguous_row_types(library, analysis, diagnostics);
    for query in &queries {
        query::validate_row_type_is_mapped(query, row_columns, &ambiguous, diagnostics);
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
    // A name that means two different row classes has no one column set to
    // check against, so those queries are described without a row check rather
    // than checked against whichever declaration happened to win.
    let checkable = row_columns
        .iter()
        .filter(|(name, _)| !ambiguous.contains(*name))
        .map(|(name, columns)| (name.clone(), columns.clone()))
        .collect::<HashMap<_, _>>();
    sqlx::validate_sqlx_describe(library, db, &queries, &checkable, options, diagnostics);
}

/// Reports row class names a package declares twice, returning the ambiguous set.
///
/// A `queryAs<T>` type argument is a bare name, so two libraries each declaring
/// an `Order` leave no way to know which one a query means. Reported once per
/// package, from the library that declares the first one by path.
fn report_ambiguous_row_types(
    library: &DartFileIr,
    analysis: &WorkspaceAnalysis,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashSet<String> {
    let duplicates = duplicate_row_types(analysis, &library.package_name);
    let mut ambiguous = HashSet::new();
    for (name, paths) in duplicates {
        if paths
            .first()
            .is_some_and(|first| first == &library.source_path)
        {
            diagnostics.push(Diagnostic::error(format!(
                "Package `{}` declares more than one row class named `{name}` ({}), so a \
                 `queryAs<{name}>` has no one row type to be checked against. Rename one, or \
                 keep one row class per name in a package.",
                library.package_name,
                paths
                    .iter()
                    .map(|path| format!("`{path}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        ambiguous.insert(name);
    }
    ambiguous
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
