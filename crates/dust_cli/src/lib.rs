#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Thin command-line interface for the Dust driver."]

mod args;
mod exit_code;
mod render;
mod run;
mod terminal;

pub use args::{CliCommand, CliOptions, ParsedCli, parse_cli_args};
pub use run::{CliRun, run_cli, run_from_env};
