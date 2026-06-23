/// Legacy database annotation name.
pub(crate) const DATABASE: &str = "Database";
/// SQLx-style database annotation name.
pub(crate) const SQLX_DATABASE: &str = "SqlxDatabase";
/// Legacy DAO annotation name.
pub(crate) const DAO: &str = "Dao";
/// SQLx-style DAO annotation name.
pub(crate) const SQLX_DAO: &str = "SqlxDao";
/// Method annotation for SQL queries.
pub(crate) const QUERY: &str = "Query";
/// Row-mapper trait annotation name.
pub(crate) const FROM_ROW: &str = "FromRow";
/// Field-level row mapping annotation name.
pub(crate) const SQLX: &str = "Sqlx";
/// Fully qualified `FromRow` trait symbol.
pub(crate) const FROM_ROW_SYMBOL: &str = "dust_dart::FromRow";

/// Trait symbols claimed by the DB row mapper.
pub(crate) const CLAIMED_TRAIT_SYMBOLS: &[&str] = &[FROM_ROW_SYMBOL];

/// Fully qualified config symbols claimed in full database mode.
pub(crate) const CLAIMED_DATABASE_CONFIG_SYMBOLS: &[&str] = &[
    "dust_dart::Database",
    "dust_dart::SqlxDatabase",
    "dust_dart::Dao",
    "dust_dart::SqlxDao",
    "dust_dart::Query",
    "dust_dart::Sqlx",
];

/// Fully qualified config symbols claimed in row-mapper-only mode.
pub(crate) const CLAIMED_ROW_CONFIG_SYMBOLS: &[&str] = &["dust_dart::Sqlx"];

/// Short annotation names supported in full database mode.
pub(crate) const SUPPORTED_DATABASE_ANNOTATIONS: &[&str] =
    &[DATABASE, SQLX_DATABASE, DAO, SQLX_DAO, QUERY, FROM_ROW];

/// Short annotation names supported in row-mapper-only mode.
pub(crate) const SUPPORTED_ROW_ANNOTATIONS: &[&str] = &[FROM_ROW];
