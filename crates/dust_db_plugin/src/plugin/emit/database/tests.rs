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
  Future<Result<Unit, SqlxError>> migrate() => Future<Result<Unit, SqlxError>>.value(const Ok(unit));

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
  Future<Result<Unit, SqlxError>> migrate() => _driver.migrate();

  @override
  UnsafeSql get unsafe => PostgresUnsafeSql(_driver);

  Pool get pool => _driver;
}

const Map<String, String> _$appDatabaseMigrations = <String, String>{};"#;
