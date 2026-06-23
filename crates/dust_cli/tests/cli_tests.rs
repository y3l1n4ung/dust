//! Integration test harness for the Dust CLI binary and command lifecycle.

#[path = "cli_tests/binary.rs"]
mod binary;

#[path = "cli_tests/build.rs"]
mod build;

#[path = "cli_tests/diagnostics.rs"]
mod diagnostics;

#[path = "cli_tests/helpers.rs"]
mod helpers;

#[path = "cli_tests/lifecycle.rs"]
mod lifecycle;
