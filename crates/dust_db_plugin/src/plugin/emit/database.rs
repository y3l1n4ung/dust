use std::{fs, path::Path};

use dust_dart_emit::render_template;
use dust_ir::DartFileIr;
use serde::Serialize;

use crate::plugin::{migrations::applied_migration_files, model::DatabaseClass};

use super::shared::{escape_dart_string, lower_first};

/// Template context for a generated database implementation class.
#[derive(Serialize)]
struct DatabaseContext<'a> {
    /// Generated private implementation class name.
    generated_name: &'a str,
    /// Source database interface class name.
    class_name: &'a str,
    /// Dart expression used to open the pool.
    open_expr: String,
    /// Constructor name the runtime type offers.
    factory: &'a str,
    /// Parameter that constructor takes.
    factory_parameter: &'a str,
    /// Dart type carrying per-connection settings.
    options_type: &'a str,
    /// Concrete driver type the facade holds.
    ///
    /// The facade keeps the driver rather than a `Connection` so that
    /// `unsafe` needs no cast, and so that a handler holding an executor has no
    /// route to it.
    driver_type: &'a str,
    /// Dart expression producing the unchecked SQL escape hatch.
    unsafe_expr: &'a str,
    /// Dart expression applying migrations.
    migrate_expr: &'a str,
    /// Rendered migrations constant.
    migrations: String,
}

/// Template context for generated migration map constants.
#[derive(Serialize)]
struct MigrationsContext<'a> {
    /// Constant name for the migration map.
    name: &'a str,
    /// Rendered migration entries.
    entries: String,
}

/// Renders a generated database implementation class.
pub(super) fn render_database_class(library: &DartFileIr, db: &DatabaseClass<'_>) -> String {
    let class_name = &db.class.name;
    let generated_name = format!("_${class_name}");
    let migrations_name = format!("_${}Migrations", lower_first(class_name));
    // Everything dialect-specific comes from one place, so adding a database
    // is a new `Dialect` rather than another arm here.
    let dialect = db.driver.dialect();
    let open_expr = format!(
        "{}.{}(\n      {},\n      migrations: {migrations_name},\n      options: options,\n    )",
        dialect.runtime_type,
        dialect.factory,
        dialect
            .factory_parameter
            .rsplit(' ')
            .next()
            .unwrap_or("path"),
    );
    let unsafe_expr = format!("{}(_driver)", dialect.unsafe_type);
    let migrations = render_migrations_map(library, &db.migrations, &migrations_name);

    render_template(
        "database_class",
        include_str!("templates/database_class.jinja"),
        DatabaseContext {
            generated_name: &generated_name,
            class_name,
            open_expr,
            factory: dialect.factory,
            factory_parameter: dialect.factory_parameter,
            options_type: dialect.options_type,
            driver_type: dialect.runtime_type,
            unsafe_expr: &unsafe_expr,
            migrate_expr: dialect.migrate_expr,
            migrations,
        },
    )
}

/// Renders a deterministic migration map from SQL files on disk.
fn render_migrations_map(library: &DartFileIr, migrations: &str, name: &str) -> String {
    let path = Path::new(&library.package_root).join(migrations);
    let files = applied_migration_files(&path).unwrap_or_default();

    let entries = files
        .iter()
        .filter_map(|file| {
            let source = fs::read_to_string(&file.path).ok()?;
            Some(format!(
                "  '{}': '{}',",
                escape_dart_string(&file.name),
                escape_dart_string(&source)
            ))
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return render_template(
            "migrations_empty",
            include_str!("templates/migrations_empty.jinja"),
            MigrationsContext {
                name,
                entries: String::new(),
            },
        );
    }
    render_template(
        "migrations_map",
        include_str!("templates/migrations_map.jinja"),
        MigrationsContext {
            name,
            entries: entries.join("\n"),
        },
    )
}

#[cfg(test)]
#[path = "database/tests.rs"]
mod tests;
