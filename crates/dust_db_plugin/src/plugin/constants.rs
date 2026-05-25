pub(crate) const DUST_DB: &str = "DustDb";
pub(crate) const QUERY: &str = "Query";
pub(crate) const TRANSACTION: &str = "Transaction";
pub(crate) const FROM_ROW: &str = "FromRow";
pub(crate) const SQLX: &str = "Sqlx";

pub(crate) const CLAIMED_CONFIG_SYMBOLS: &[&str] = &[
    "dust_db::DustDb",
    "dust_db::Query",
    "dust_db::Transaction",
    "dust_db::FromRow",
    "dust_db::Sqlx",
];

pub(crate) const SUPPORTED_ANNOTATIONS: &[&str] = &[DUST_DB, QUERY, TRANSACTION, FROM_ROW, SQLX];
