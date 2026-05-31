pub(crate) const DATABASE: &str = "Database";
pub(crate) const SQLX_DATABASE: &str = "SqlxDatabase";
pub(crate) const DAO: &str = "Dao";
pub(crate) const SQLX_DAO: &str = "SqlxDao";
pub(crate) const QUERY: &str = "Query";
pub(crate) const FROM_ROW: &str = "FromRow";
pub(crate) const SQLX: &str = "Sqlx";
pub(crate) const FROM_ROW_SYMBOL: &str = "dust_db_annotation::FromRow";

pub(crate) const CLAIMED_TRAIT_SYMBOLS: &[&str] = &[FROM_ROW_SYMBOL];

pub(crate) const CLAIMED_DATABASE_CONFIG_SYMBOLS: &[&str] = &[
    "dust_db_annotation::Database",
    "dust_db_annotation::SqlxDatabase",
    "dust_db_annotation::Dao",
    "dust_db_annotation::SqlxDao",
    "dust_db_annotation::Query",
    "dust_db_annotation::Sqlx",
];

pub(crate) const CLAIMED_ROW_CONFIG_SYMBOLS: &[&str] = &["dust_db_annotation::Sqlx"];

pub(crate) const SUPPORTED_DATABASE_ANNOTATIONS: &[&str] =
    &[DATABASE, SQLX_DATABASE, DAO, SQLX_DAO, QUERY, FROM_ROW];

pub(crate) const SUPPORTED_ROW_ANNOTATIONS: &[&str] = &[FROM_ROW];
