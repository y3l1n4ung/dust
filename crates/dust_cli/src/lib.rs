#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Thin command-line interface for the Dust driver."]

/// Command-line argument parsing.
mod args;
/// Process exit code selection.
mod exit_code;
/// Human-readable command result rendering.
mod render;
/// CLI command execution.
mod run;
/// Interactive terminal progress rendering.
mod terminal;

pub use args::{CliCommand, CliOptions, ParsedCli, parse_cli_args, parse_cli_from_env};
pub use run::{CliRun, run_cli, run_from_env};
