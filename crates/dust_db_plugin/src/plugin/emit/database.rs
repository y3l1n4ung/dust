use std::{fs, path::Path};

use dust_ir::LibraryIr;

use crate::plugin::model::{DatabaseClass, DbDriver};

use super::shared::{escape_dart_string, lower_first};

pub(super) fn render_database_class(library: &LibraryIr, db: &DatabaseClass<'_>) -> String {
    let class_name = &db.class.name;
    let generated_name = format!("_${class_name}");
    let migrations_name = format!("_${}Migrations", lower_first(class_name));
    let open_expr = match db.driver {
        DbDriver::Sqlite3 => {
            format!("Sqlite3Driver.open(\n      path,\n      migrations: {migrations_name},\n    )")
        }
        DbDriver::Postgres => {
            "throw UnsupportedError('Driver.postgres is not supported in Dust DB v1')".to_owned()
        }
    };
    let migrations = render_migrations_map(library, &db.migrations, &migrations_name);

    format!(
        "final class {generated_name} implements {class_name} {{\n  {generated_name}._(this.pool);\n\n  factory {generated_name}.open(String path) {{\n    final pool = {open_expr};\n    return {generated_name}._(pool);\n  }}\n\n  @override\n  final Pool pool;\n}}\n\n{migrations}"
    )
}

fn render_migrations_map(library: &LibraryIr, migrations: &str, name: &str) -> String {
    let path = Path::new(&library.package_root).join(migrations);
    let mut files = fs::read_dir(&path)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("sql"))
        .collect::<Vec<_>>();
    files.sort();

    let entries = files
        .iter()
        .filter_map(|file| {
            let source = fs::read_to_string(file).ok()?;
            let key = file.file_name()?.to_str()?;
            Some(format!(
                "  '{}': '{}',",
                escape_dart_string(key),
                escape_dart_string(&source)
            ))
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return format!("const Map<String, String> {name} = <String, String>{{}};");
    }
    format!(
        "const Map<String, String> {name} = <String, String>{{\n{}\n}};",
        entries.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use dust_ir::{ClassIr, ClassKindIr, LibraryIr, SpanIr};
    use dust_text::{FileId, TextRange};

    use super::*;

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

    fn library(root: &std::path::Path, classes: Vec<ClassIr>) -> LibraryIr {
        LibraryIr {
            package_root: root.display().to_string(),
            package_name: "emit_test".to_owned(),
            source_path: "lib/db.dart".to_owned(),
            output_path: "lib/db.g.dart".to_owned(),
            imports: Vec::new(),
            span: span(),
            classes,
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
    fn emits_postgres_database_as_v1_unsupported() {
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
  _$AppDatabase._(this.pool);

  factory _$AppDatabase.open(String path) {
    final pool = Sqlite3Driver.open(
      path,
      migrations: _$appDatabaseMigrations,
    );
    return _$AppDatabase._(pool);
  }

  @override
  final Pool pool;
}

const Map<String, String> _$appDatabaseMigrations = <String, String>{
  '001_schema.sql': 'CREATE TABLE logs(message TEXT);\n',
  '002_quote.sql': 'INSERT INTO logs(message) VALUES(\'cost \$1\');\n',
};"#;

    const EXPECTED_POSTGRES_DATABASE: &str = r#"final class _$AppDatabase implements AppDatabase {
  _$AppDatabase._(this.pool);

  factory _$AppDatabase.open(String path) {
    final pool = throw UnsupportedError('Driver.postgres is not supported in Dust DB v1');
    return _$AppDatabase._(pool);
  }

  @override
  final Pool pool;
}

const Map<String, String> _$appDatabaseMigrations = <String, String>{};"#;
}
