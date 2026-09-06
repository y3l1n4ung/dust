//! What the engine knows about one database, in one place.
//!
//! Emit, validation, and the escape hatch each used to `match` on the driver at
//! the point of use, so adding a database meant finding every arm and missing
//! one meant generated code naming a type from the wrong driver. A dialect is a
//! value instead.

use super::model::DbDriver;

/// One database, and the Dart runtime that talks to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Dialect {
    /// The driver this describes.
    pub(crate) driver: DbDriver,
    /// Stable name used in analysis keys and query cache entries.
    pub(crate) name: &'static str,
    /// Dart type the generated facade holds.
    pub(crate) runtime_type: &'static str,
    /// Constructor the runtime type offers.
    pub(crate) factory: &'static str,
    /// Parameter that constructor takes, as it appears in Dart source.
    ///
    /// SQLite opens a file and Postgres connects to a server, and the facade's
    /// signature follows the database rather than pretending they are the same.
    pub(crate) factory_parameter: &'static str,
    /// Dart type carrying per-connection settings.
    pub(crate) options_type: &'static str,
    /// Dart type implementing the unchecked SQL escape hatch.
    pub(crate) unsafe_type: &'static str,
    /// Dart expression applying migrations from the generated facade.
    pub(crate) migrate_expr: &'static str,
    /// Whether the driver rewrites `$n` into another placeholder form.
    ///
    /// SQLite has no `$n`, so its driver rewrites to `?` at bind time and a
    /// repeated `$1` becomes two binds. PostgreSQL reads `$n` itself and binds
    /// a repeated one once. The engine has to describe the text the database
    /// will actually receive, and expect the bind count that dialect implies.
    pub(crate) rewrites_placeholders: bool,
    /// Whether the dialect's nullability inference is worth reporting.
    ///
    /// SQLite describes a `PRIMARY KEY` column as nullable, so checking its
    /// inference warns about correct code: `fixtures/server_app` produced five
    /// such warnings, every one of them wrong. PostgreSQL describes the same
    /// shapes accurately. The column-alias overrides are what a dialect with
    /// untrustworthy inference leaves callers, and they work either way.
    pub(crate) checks_nullability: bool,
    /// URL schemes `DUST_DATABASE_URL` may use for this dialect.
    ///
    /// One workspace can hold projects on both drivers, and one environment
    /// variable names one database. Without this, a SQLite project handed a
    /// PostgreSQL URL tries to open it as a file and reports whatever the
    /// other driver's URL parser disliked.
    pub(crate) url_schemes: &'static [&'static str],
    /// Whether SQL for this dialect is checked against the schema at build time.
    ///
    /// False means the runtime works but `describe` does not, so queries reach
    /// the database unchecked and the build warns rather than refusing.
    pub(crate) validates: bool,
}

/// SQLite through `package:sqlite3`.
const SQLITE3: Dialect = Dialect {
    driver: DbDriver::Sqlite3,
    name: "sqlite3",
    runtime_type: "Sqlite3Driver",
    factory: "open",
    factory_parameter: "String path",
    options_type: "SqliteConnectOptions",
    unsafe_type: "Sqlite3UnsafeSql",
    // Applied while opening, so there is nothing left to do.
    migrate_expr: "Future<Result<Unit, SqlxError>>.value(const Ok(unit))",
    rewrites_placeholders: true,
    checks_nullability: false,
    url_schemes: &["sqlite"],
    validates: true,
};

/// PostgreSQL through `package:postgres`.
const POSTGRES: Dialect = Dialect {
    driver: DbDriver::Postgres,
    name: "postgres",
    runtime_type: "PostgresDriver",
    factory: "connect",
    factory_parameter: "String url",
    options_type: "PgConnectOptions",
    unsafe_type: "PostgresUnsafeSql",
    migrate_expr: "_driver.migrate()",
    rewrites_placeholders: false,
    checks_nullability: true,
    url_schemes: &["postgres", "postgresql"],
    validates: true,
};

impl DbDriver {
    /// Returns everything the engine knows about this driver.
    pub(crate) const fn dialect(self) -> &'static Dialect {
        match self {
            Self::Sqlite3 => &SQLITE3,
            Self::Postgres => &POSTGRES,
        }
    }

    /// Returns the stable driver name used in analysis keys and cache entries.
    pub(crate) const fn as_str(self) -> &'static str {
        self.dialect().name
    }
}

impl Dialect {
    /// Whether `url` names a database this dialect can validate against.
    ///
    /// A URL with no scheme is a bare SQLite path, which only SQLite accepts.
    pub(crate) fn accepts_url(&self, url: &str) -> bool {
        match url.split_once("://") {
            Some((scheme, _)) => self
                .url_schemes
                .iter()
                .any(|known| scheme.eq_ignore_ascii_case(known)),
            None => self.driver == DbDriver::Sqlite3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A copied dialect that kept another driver's type names would emit code
    /// naming the wrong package, and nothing else would catch it.
    #[test]
    fn every_driver_names_a_distinct_runtime() {
        let dialects = [DbDriver::Sqlite3.dialect(), DbDriver::Postgres.dialect()];
        for (index, left) in dialects.iter().enumerate() {
            for right in dialects.iter().skip(index + 1) {
                assert_ne!(left.name, right.name);
                assert_ne!(left.runtime_type, right.runtime_type);
                assert_ne!(left.unsafe_type, right.unsafe_type);
                assert_ne!(left.migrate_expr, right.migrate_expr);
                assert_ne!(left.options_type, right.options_type);
            }
        }
    }

    /// One `DUST_DATABASE_URL` and two drivers in a workspace: each project
    /// has to recognise the URL that is not for it, rather than trying to open
    /// it and reporting the other driver's parse error.
    #[test]
    fn a_dialect_only_accepts_its_own_urls() {
        let sqlite = DbDriver::Sqlite3.dialect();
        let postgres = DbDriver::Postgres.dialect();

        assert!(sqlite.accepts_url("sqlite://app.db"));
        assert!(sqlite.accepts_url("app.db"));
        assert!(!sqlite.accepts_url("postgres://user@localhost/app?sslmode=disable"));

        assert!(postgres.accepts_url("postgres://user@localhost/app"));
        assert!(postgres.accepts_url("POSTGRESQL://user@localhost/app"));
        assert!(!postgres.accepts_url("sqlite://app.db"));
        assert!(!postgres.accepts_url("app.db"));
    }

    #[test]
    fn driver_names_round_trip_through_analysis() {
        assert_eq!(DbDriver::Sqlite3.as_str(), "sqlite3");
        assert_eq!(DbDriver::Postgres.as_str(), "postgres");
    }
}
