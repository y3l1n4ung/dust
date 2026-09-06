//! What the engine knows about one database, in one place.
//!
//! Everything dialect-specific used to be a `match` at the point of use: emit
//! matched to pick a runtime type, validation matched to decide whether to run
//! at all, and a third match decided the escape hatch. Adding a database meant
//! finding all of them, and missing one meant generated code that named a type
//! from the wrong driver.
//!
//! A dialect is a value instead. Adding MySQL is one more `Dialect`, and the
//! compiler names every place that has to answer for it.

use super::model::DbDriver;

/// One database, and the Dart runtime that talks to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Dialect {
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
    /// Whether SQL for this dialect is checked against the schema at build time.
    ///
    /// False means the runtime works but `describe` is not wired up, so queries
    /// reach the database unchecked and the build says so.
    pub(crate) validates: bool,
}

/// SQLite through `package:sqlite3`.
const SQLITE3: Dialect = Dialect {
    name: "sqlite3",
    runtime_type: "Sqlite3Driver",
    factory: "open",
    factory_parameter: "String path",
    options_type: "SqliteConnectOptions",
    unsafe_type: "Sqlite3UnsafeSql",
    validates: true,
};

/// PostgreSQL through `package:postgres`.
const POSTGRES: Dialect = Dialect {
    name: "postgres",
    runtime_type: "PgPool",
    factory: "connect",
    factory_parameter: "String url",
    options_type: "PgConnectOptions",
    unsafe_type: "PostgresUnsafeSql",
    // `describe` needs a live server and the sqlx Postgres backend, neither of
    // which is wired up yet.
    validates: false,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_driver_names_a_distinct_runtime() {
        // A copied `Dialect` that kept another driver's type names would emit
        // code naming the wrong package, and nothing else would catch it.
        let dialects = [DbDriver::Sqlite3.dialect(), DbDriver::Postgres.dialect()];
        for (index, left) in dialects.iter().enumerate() {
            for right in dialects.iter().skip(index + 1) {
                assert_ne!(left.name, right.name);
                assert_ne!(left.runtime_type, right.runtime_type);
                assert_ne!(left.unsafe_type, right.unsafe_type);
                assert_ne!(left.options_type, right.options_type);
            }
        }
    }

    #[test]
    fn driver_names_round_trip_through_analysis() {
        assert_eq!(DbDriver::Sqlite3.as_str(), "sqlite3");
        assert_eq!(DbDriver::Postgres.as_str(), "postgres");
    }
}
