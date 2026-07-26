/// Resolver-normalized database configuration attached to a class, method, or
/// field annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbConfigIr {
    /// Options from a `Database` or `SqlxDatabase` annotation.
    Database(DbDatabaseConfigIr),
    /// Row mapping options from a `Sqlx` annotation.
    Sqlx(DbSqlxConfigIr),
    /// SQL source from a `Query` annotation.
    Query(DbQueryConfigIr),
    /// Marker-only database configuration such as `Dao` or `SqlxDao`.
    Marker,
}

/// Options from a database annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbDatabaseConfigIr {
    /// Selected database driver.
    pub driver: DbDriverIr,
    /// Migration directory configured by the annotation.
    pub migrations: String,
}

/// Database driver selected by a normalized database annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbDriverIr {
    /// SQLite through the `dust_db_sqlite3` runtime.
    Sqlite3,
    /// PostgreSQL placeholder driver.
    Postgres,
}

/// Row mapping options from a `Sqlx` annotation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DbSqlxConfigIr {
    /// Explicit SQL column name override.
    pub rename: Option<String>,
    /// Bulk rename rule applied to row field names.
    pub rename_all: Option<DbRenameRuleIr>,
    /// Whether this field flattens another row-mapped value.
    pub flatten: bool,
    /// Default expression used when a column is absent or null.
    pub default_value_source: Option<String>,
    /// Whether this field is skipped by row mapping.
    pub skip: bool,
    /// Whether this field is decoded from JSON.
    pub json: bool,
    /// Source expression for try-conversion after row decoding.
    pub try_from_source: Option<String>,
    /// Converter class name referenced by `tryFrom`, when it can be resolved.
    pub try_from_class_name: Option<String>,
}

/// SQLx row field rename strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbRenameRuleIr {
    /// Lowercase field names.
    Lower,
    /// Uppercase field names.
    Upper,
    /// PascalCase field names.
    Pascal,
    /// camelCase field names.
    Camel,
    /// snake_case field names.
    Snake,
    /// SCREAMING_SNAKE_CASE field names.
    ScreamingSnake,
    /// kebab-case field names.
    Kebab,
    /// SCREAMING-KEBAB-CASE field names.
    ScreamingKebab,
}

/// SQL source from a `Query` annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbQueryConfigIr {
    /// SQL source, or an empty string for a dynamic expression.
    pub sql: String,
    /// Whether the SQL source was a static string literal.
    pub sql_source_static: bool,
}
