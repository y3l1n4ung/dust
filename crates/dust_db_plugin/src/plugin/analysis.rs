//! Package-wide DB facts collected during the shared workspace scan.
//!
//! Validation needs the schema and the row classes, and a real project keeps
//! the database class, the row classes, and the queries in different files.
//! The scan phase visits every library before any of them is validated, so
//! that is where the facts are gathered; each library then validates against
//! the whole package rather than against itself.
//!
//! The analysis store carries string sets, so each fact is packed into one
//! value and unpacked on read. Every value leads with the package name because
//! the store is shared across a workspace and two packages may each declare a
//! row class called `Order`.

use std::collections::{HashMap, HashSet};

use dust_ir::DartFileIr;
use dust_plugin_api::{WorkspaceAnalysis, WorkspaceAnalysisBuilder};

use super::{
    model::DbDriver,
    parse::{database_classes, effective_column_name, row_classes, sqlx_config},
};

/// Database classes declared anywhere in the package.
pub(crate) const DATABASES_KEY: &str = "dust_db_plugin.databases.v1";
/// Row classes and the columns each one requires.
pub(crate) const ROW_COLUMNS_KEY: &str = "dust_db_plugin.row_columns.v1";

/// Separates the fields packed into one analysis value.
const UNIT: char = '\u{1f}';
/// Marks a column a row class reads directly.
const COLUMN: &str = "c:";
/// Marks a row class flattened into another one.
const FLATTEN: &str = "f:";

/// One `@SqlxDatabase` class found somewhere in the package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackageDatabase {
    /// Declaring class name, used in diagnostics and to order duplicates.
    pub(crate) name: String,
    /// Target database driver.
    pub(crate) driver: DbDriver,
    /// Migration directory as written in the annotation.
    pub(crate) migrations: String,
}

/// Collects the package-wide DB facts contributed by one library.
pub(crate) fn collect_db_workspace_analysis(
    library: &DartFileIr,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    let package = library.package_name.as_str();
    for db in database_classes(library) {
        let driver = match db.driver {
            DbDriver::Sqlite3 => "sqlite3",
            DbDriver::Postgres => "postgres",
        };
        analysis.add_string_set_value(
            DATABASES_KEY,
            pack([package, &db.class.name, driver, &db.migrations]),
        );
    }
    for row in row_classes(library) {
        let mut fields = vec![package.to_owned(), row.class.name.clone()];
        for field in &row.class.fields {
            let config = sqlx_config(&field.configs);
            if config.skip {
                continue;
            }
            if config.flatten {
                // The flattened class may live in another library, which this
                // scan has not necessarily reached yet. Record the reference
                // and expand it once every library has contributed.
                if let Some(name) = field.ty.name() {
                    fields.push(format!("{FLATTEN}{name}"));
                }
                continue;
            }
            let column = effective_column_name(&row.config, &field.name, &config);
            fields.push(format!("{COLUMN}{column}"));
        }
        analysis.add_string_set_value(ROW_COLUMNS_KEY, fields.join(&UNIT.to_string()));
    }
}

/// Returns every `@SqlxDatabase` declared in one package, ordered by class name.
pub(crate) fn package_databases(
    analysis: &WorkspaceAnalysis,
    package: &str,
) -> Vec<PackageDatabase> {
    let mut databases = analysis
        .string_set(DATABASES_KEY)
        .unwrap_or_default()
        .iter()
        .filter_map(|value| parse_database(value, package))
        .collect::<Vec<_>>();
    databases.sort_by(|left, right| left.name.cmp(&right.name));
    databases
}

/// Returns the required column set for every row class in one package.
///
/// Flattened rows contribute their own columns, however many libraries the
/// chain crosses. A cycle stops at the class already being expanded rather
/// than recursing forever; the row validator is what reports it.
pub(crate) fn package_row_column_map(
    analysis: &WorkspaceAnalysis,
    package: &str,
) -> HashMap<String, HashSet<String>> {
    let declared = analysis
        .string_set(ROW_COLUMNS_KEY)
        .unwrap_or_default()
        .iter()
        .filter_map(|value| parse_row(value, package))
        .collect::<HashMap<_, _>>();
    declared
        .keys()
        .map(|name| {
            let mut columns = HashSet::new();
            let mut seen = HashSet::new();
            expand_row(name, &declared, &mut columns, &mut seen);
            (name.clone(), columns)
        })
        .collect()
}

/// One row class's own columns and the rows flattened into it.
type DeclaredRow = (Vec<String>, Vec<String>);

/// Adds one row class's columns, following flattened rows.
fn expand_row(
    name: &str,
    declared: &HashMap<String, DeclaredRow>,
    columns: &mut HashSet<String>,
    seen: &mut HashSet<String>,
) {
    if !seen.insert(name.to_owned()) {
        return;
    }
    let Some((own, flattened)) = declared.get(name) else {
        return;
    };
    columns.extend(own.iter().cloned());
    for row in flattened {
        expand_row(row, declared, columns, seen);
    }
}

/// Packs analysis fields into one string-set value.
fn pack<'a>(fields: impl IntoIterator<Item = &'a str>) -> String {
    fields
        .into_iter()
        .collect::<Vec<_>>()
        .join(&UNIT.to_string())
}

/// Reads one packed database value belonging to `package`.
fn parse_database(value: &str, package: &str) -> Option<PackageDatabase> {
    let mut fields = value.split(UNIT);
    if fields.next()? != package {
        return None;
    }
    let name = fields.next()?.to_owned();
    let driver = match fields.next()? {
        "sqlite3" => DbDriver::Sqlite3,
        "postgres" => DbDriver::Postgres,
        _ => return None,
    };
    let migrations = fields.next()?.to_owned();
    Some(PackageDatabase {
        name,
        driver,
        migrations,
    })
}

/// Reads one packed row value belonging to `package`.
fn parse_row(value: &str, package: &str) -> Option<(String, DeclaredRow)> {
    let mut fields = value.split(UNIT);
    if fields.next()? != package {
        return None;
    }
    let name = fields.next()?.to_owned();
    let mut columns = Vec::new();
    let mut flattened = Vec::new();
    for field in fields {
        if let Some(column) = field.strip_prefix(COLUMN) {
            columns.push(column.to_owned());
        } else if let Some(row) = field.strip_prefix(FLATTEN) {
            flattened.push(row.to_owned());
        }
    }
    Some((name, (columns, flattened)))
}

#[cfg(test)]
#[path = "analysis/tests.rs"]
/// Unit tests for package-wide DB analysis packing.
mod tests;
