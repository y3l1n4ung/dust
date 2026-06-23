use std::collections::HashSet;

use dust_ir::DartFileIr;
use dust_plugin_api::PluginContribution;

use super::{
    DbPluginOptions,
    parse::{dao_classes, database_classes, imported_row_names, row_classes},
};

/// Renders generated DAO methods.
mod dao;
/// Renders generated database openers and migrations.
mod database;
/// Renders generated `FromRow` extensions.
mod row;
/// Shared Dart rendering helpers for DB generation.
mod shared;

use self::{
    dao::render_dao_class, database::render_database_class, row::render_from_row_extension,
};

/// Emits DB plugin generated sections for a Dart library.
pub(crate) fn emit_db_library(
    library: &DartFileIr,
    options: DbPluginOptions,
) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let mut sections = Vec::new();
    let rows = row_classes(library);

    let databases = options.databases.then(|| database_classes(library));
    let daos = options.databases.then(|| dao_classes(library));
    if options.databases
        && databases.as_ref().is_none_or(Vec::is_empty)
        && daos.as_ref().is_none_or(Vec::is_empty)
    {
        return contribution;
    }

    for row in &rows {
        sections.push(render_from_row_extension(library, row.class, &row.config));
    }

    if options.databases {
        let databases = databases.expect("database classes are collected in database mode");
        let driver = databases
            .first()
            .map_or(super::model::DbDriver::Sqlite3, |db| db.driver);
        for db in databases {
            sections.push(render_database_class(library, &db));
        }
        let imported_rows = imported_row_names(library);
        let row_names = rows
            .iter()
            .map(|row| row.class.name.as_str())
            .chain(imported_rows.iter().map(String::as_str))
            .collect::<HashSet<_>>();
        for dao in daos.expect("DAO classes are collected in database mode") {
            sections.push(render_dao_class(&dao, &row_names, driver));
        }
    }

    if !sections.is_empty() {
        contribution.support_types.push(sections.join("\n\n"));
    }

    contribution
}
