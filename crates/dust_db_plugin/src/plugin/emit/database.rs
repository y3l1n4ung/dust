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
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use dust_ir::{ClassIr, ClassKindIr, DartFileIr, SpanIr};
    use dust_text::{FileId, TextRange};

    use super::*;
    use crate::plugin::model::DbDriver;

    fn span() -> SpanIr {
        SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
    }

    fn class(name: &str) -> ClassIr {
        ClassIr {
            kind: ClassKindIr::Class,
            name: name.to_owned(),
            is_abstract: true,
            is_interface: false,
            superclass_name: None,
            span: span(),
            fields: Vec::new(),
            constructors: Vec::new(),
            methods: Vec::new(),
            traits: Vec::new(),
            configs: Vec::new(),
            serde: None,
        }
    }

    fn library(root: &std::path::Path, classes: Vec<ClassIr>) -> DartFileIr {
        DartFileIr {
            package_root: root.display().to_string(),
            package_name: "emit_test".to_owned(),
            source_path: "lib/db.dart".to_owned(),
            output_path: "lib/db.g.dart".to_owned(),
            imports: Vec::new(),
            library: None,
            library_annotations: Vec::new(),
            import_directives: Vec::new(),
            export_directives: Vec::new(),
            part_directives: Vec::new(),
            part_of: None,
            span: span(),
            classes,
            mixins: Vec::new(),
            extensions: Vec::new(),
            extension_types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
            enums: Vec::new(),
            query_calls: Vec::new(),
        }
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("dust_db_emit_{name}_{stamp}"))
    }

    #[test]
    fn emits_database_class_with_sorted_escaped_migrations() {
        let root = temp_root("migrations");
        let migrations = root.join("migrations");
        fs::create_dir_all(&migrations).unwrap();
        fs::write(
            migrations.join("002_quote.sql"),
            "INSERT INTO logs(message) VALUES('cost $1');\n",
        )
        .unwrap();
        fs::write(
            migrations.join("003_reversible.up.sql"),
            "ALTER TABLE logs ADD COLUMN tag TEXT;\n",
        )
        .unwrap();
        fs::write(
            migrations.join("003_reversible.down.sql"),
            "ALTER TABLE logs DROP COLUMN tag;\n",
        )
        .unwrap();
        fs::write(
            migrations.join("001_schema.sql"),
            "CREATE TABLE logs(message TEXT);\n",
        )
        .unwrap();

        let db_class = class("AppDatabase");
        let library = library(&root, vec![db_class.clone()]);
        let db = DatabaseClass {
            class: &db_class,
            driver: DbDriver::Sqlite3,
            migrations: "migrations".to_owned(),
        };

        assert_eq!(
            render_database_class(&library, &db),
            EXPECTED_SQLITE_DATABASE
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn emits_postgres_database_against_its_own_runtime() {
        let db_class = class("AppDatabase");
        let library = library(std::path::Path::new(""), vec![db_class.clone()]);
        let db = DatabaseClass {
            class: &db_class,
            driver: DbDriver::Postgres,
            migrations: "migrations".to_owned(),
        };

        assert_eq!(
            render_database_class(&library, &db),
            EXPECTED_POSTGRES_DATABASE
        );
    }

    const EXPECTED_SQLITE_DATABASE: &str = r#"final class _$AppDatabase implements AppDatabase {
  _$AppDatabase._(this._driver);

  factory _$AppDatabase.open(
    String path, {
    SqliteConnectOptions? options,
  }) {
    final driver = Sqlite3Driver.open(
      path,
      migrations: _$appDatabaseMigrations,
      options: options,
    );
    return _$AppDatabase._(driver);
  }

  final Sqlite3Driver _driver;

  @override
  Connection get connection => _driver;

  @override
  UnsafeSql get unsafe => Sqlite3UnsafeSql(_driver);

  Pool get pool => _driver;
}

const Map<String, String> _$appDatabaseMigrations = <String, String>{
  '001_schema.sql': 'CREATE TABLE logs(message TEXT);\n',
  '002_quote.sql': 'INSERT INTO logs(message) VALUES(\'cost \$1\');\n',
  '003_reversible.up.sql': 'ALTER TABLE logs ADD COLUMN tag TEXT;\n',
};"#;

    /// Each dialect names its own runtime, and nothing of the other leaks in.
    ///
    /// The facade signature follows the database rather than pretending they
    /// are alike: SQLite opens a path, PostgreSQL connects to a URL.
    const EXPECTED_POSTGRES_DATABASE: &str = r#"final class _$AppDatabase implements AppDatabase {
  _$AppDatabase._(this._driver);

  factory _$AppDatabase.connect(
    String url, {
    PgConnectOptions? options,
  }) {
    final driver = PostgresDriver.connect(
      url,
      migrations: _$appDatabaseMigrations,
      options: options,
    );
    return _$AppDatabase._(driver);
  }

  final PostgresDriver _driver;

  @override
  Connection get connection => _driver;

  @override
  UnsafeSql get unsafe => PostgresUnsafeSql(_driver);

  Pool get pool => _driver;
}

const Map<String, String> _$appDatabaseMigrations = <String, String>{};"#;
}
