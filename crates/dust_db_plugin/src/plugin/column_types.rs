//! Which described SQL types a Dart field may read.
//!
//! Describe reports the type the database inferred for each column. Comparing
//! that to the field reading it catches a `TEXT` column decoded into an `int`,
//! which otherwise fails at run time on the row that reaches it.
//!
//! The table is deliberately permissive, and the findings are warnings. SQLite
//! has type affinity rather than types: a column declared `NUMERIC` holds
//! whatever was written to it, an expression has no declared type at all, and a
//! strict table would reject working code. PostgreSQL is stricter, so its rows
//! are narrower. Anything not listed is accepted rather than reported, so a type
//! this table has not learned yet costs nothing.

use super::model::DbDriver;

/// Whether [dart_type] may read a column the database described as [sql_type].
///
/// Returns true when the pair is accepted, when either side is unknown, or when
/// the dialect has no table — the check reports what it is sure about and stays
/// quiet otherwise.
pub(crate) fn accepts(driver: DbDriver, dart_type: &str, sql_type: &str) -> bool {
    if dart_type.is_empty() || sql_type.is_empty() {
        return true;
    }
    let sql = normalize(sql_type);
    let Some(accepted) = accepted_sql_types(driver, dart_type) else {
        // A Dart type this table says nothing about — a converter type, an enum
        // reached through `tryFrom` — is not something to guess at.
        return true;
    };
    accepted.iter().any(|candidate| *candidate == sql)
}

/// Returns the SQL types a Dart type may read, if the table covers it.
fn accepted_sql_types(driver: DbDriver, dart_type: &str) -> Option<&'static [&'static str]> {
    (driver.dialect().accepted_sql_types)(dart_type)
}

/// SQLite's accepted pairs.
///
/// Affinity makes these wide. `INTEGER` reads into `bool` because SQLite has no
/// boolean, and `TEXT` reads into `DateTime` because it has no date type
/// either — both are how the row adapter already behaves.
pub(crate) fn sqlite_types(dart_type: &str) -> Option<&'static [&'static str]> {
    Some(match dart_type {
        "int" => &["INTEGER", "NUMERIC", "BOOLEAN", "NULL"],
        "double" => &["REAL", "NUMERIC", "INTEGER", "NULL"],
        "num" => &["INTEGER", "REAL", "NUMERIC", "NULL"],
        "bool" => &["INTEGER", "BOOLEAN", "NUMERIC", "NULL"],
        "String" => &["TEXT", "NULL"],
        "DateTime" => &["TEXT", "DATETIME", "NULL"],
        _ => return None,
    })
}

/// PostgreSQL's accepted pairs.
///
/// Real types, so these are narrower. `numeric` is absent from every row on
/// purpose: it is arbitrary-precision and Dart has no counterpart, so which
/// Dart type may read it is still open.
pub(crate) fn postgres_types(dart_type: &str) -> Option<&'static [&'static str]> {
    Some(match dart_type {
        "int" => &[
            "INT2", "INT4", "INT8", "SMALLINT", "INTEGER", "BIGINT", "OID",
        ],
        // `NUMERIC` is arbitrary-precision and Dart has no counterpart, so which
        // Dart type should read it is still open. Reading it as a `double`
        // loses precision, but warning about a mapping nobody has decided
        // against would be a false report, so it is accepted for now.
        "double" => &[
            "FLOAT4",
            "FLOAT8",
            "REAL",
            "DOUBLE PRECISION",
            "NUMERIC",
            "DECIMAL",
        ],
        "num" => &[
            "INT2",
            "INT4",
            "INT8",
            "FLOAT4",
            "FLOAT8",
            "SMALLINT",
            "INTEGER",
            "BIGINT",
            "REAL",
            "DOUBLE PRECISION",
            "NUMERIC",
            "DECIMAL",
        ],
        "bool" => &["BOOL", "BOOLEAN"],
        "String" => &[
            "TEXT", "VARCHAR", "CHAR", "BPCHAR", "NAME", "CITEXT", "UUID", "JSON", "JSONB",
        ],
        "DateTime" => &[
            "TIMESTAMP",
            "TIMESTAMPTZ",
            "DATE",
            "TIMESTAMP WITH TIME ZONE",
            "TIMESTAMP WITHOUT TIME ZONE",
        ],
        _ => return None,
    })
}

/// Upper-cases a described type and drops any size or precision.
///
/// `VARCHAR(64)` and `NUMERIC(10, 2)` describe the same type as their bare
/// forms for this purpose.
fn normalize(sql_type: &str) -> String {
    let base = sql_type.split('(').next().unwrap_or(sql_type);
    base.trim().to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_column_does_not_read_into_an_int() {
        assert!(!accepts(DbDriver::Sqlite3, "int", "TEXT"));
        assert!(!accepts(DbDriver::Postgres, "int", "TEXT"));
    }

    #[test]
    fn sqlite_affinity_is_accepted_where_it_is_real() {
        // SQLite has no boolean and no date type; the row adapter reads an
        // integer as one and text as the other, so the table has to agree.
        assert!(accepts(DbDriver::Sqlite3, "bool", "INTEGER"));
        assert!(accepts(DbDriver::Sqlite3, "DateTime", "TEXT"));
    }

    #[test]
    fn postgres_keeps_its_real_types_apart() {
        assert!(accepts(DbDriver::Postgres, "bool", "BOOL"));
        assert!(!accepts(DbDriver::Postgres, "bool", "INT4"));
        assert!(accepts(DbDriver::Postgres, "DateTime", "TIMESTAMPTZ"));
    }

    #[test]
    fn num_accepts_either_side_of_the_integer_and_float_split() {
        // `num` is the Dart type that admits both, so every numeric column has
        // to satisfy it on either dialect. Nothing else exercised the
        // PostgreSQL half of that.
        for sql in ["INT2", "INT4", "INT8", "BIGINT", "SMALLINT"] {
            assert!(accepts(DbDriver::Postgres, "num", sql), "{sql}");
        }
        for sql in [
            "FLOAT4",
            "FLOAT8",
            "REAL",
            "DOUBLE PRECISION",
            "NUMERIC",
            "DECIMAL",
        ] {
            assert!(accepts(DbDriver::Postgres, "num", sql), "{sql}");
        }
        assert!(!accepts(DbDriver::Postgres, "num", "TEXT"));

        for sql in ["INTEGER", "REAL", "NUMERIC"] {
            assert!(accepts(DbDriver::Sqlite3, "num", sql), "{sql}");
        }
    }

    #[test]
    fn a_dart_type_the_table_says_nothing_about_is_accepted() {
        // A converter type or an enum read through `tryFrom` is not something
        // to guess at, on either dialect.
        assert!(accepts(DbDriver::Sqlite3, "Money", "TEXT"));
        assert!(accepts(DbDriver::Postgres, "Money", "TEXT"));
        assert!(accepts(DbDriver::Sqlite3, "Uint8List", "BLOB"));
    }

    #[test]
    fn size_and_precision_do_not_change_the_type() {
        assert!(accepts(DbDriver::Postgres, "String", "VARCHAR(64)"));
        assert!(accepts(DbDriver::Sqlite3, "int", "NUMERIC(10, 2)"));
    }

    #[test]
    fn an_unknown_pair_is_accepted_rather_than_guessed() {
        // A converter type, an enum read through `tryFrom`, a type the table
        // has not learned: none of these are worth a false warning.
        assert!(accepts(DbDriver::Postgres, "OrderStatus", "order_status"));
        assert!(accepts(DbDriver::Postgres, "int", ""));
        assert!(accepts(DbDriver::Sqlite3, "", "TEXT"));
    }

    #[test]
    fn postgres_numeric_is_left_undecided() {
        // Arbitrary precision with no Dart counterpart. Accepting it is the
        // quiet answer until the mapping is chosen.
        assert!(accepts(DbDriver::Postgres, "double", "NUMERIC"));
    }
}
