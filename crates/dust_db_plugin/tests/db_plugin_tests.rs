//! Integration tests for DB plugin emission and SQLx validation.

#[path = "db_plugin_tests/emission.rs"]
mod emission;
#[path = "db_plugin_tests/sqlx_validation.rs"]
mod sqlx_validation;
#[path = "db_plugin_tests/support.rs"]
mod support;
#[path = "db_plugin_tests/validation.rs"]
mod validation;
