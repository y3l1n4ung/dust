use dust_driver::CommandResult;

use crate::args::CliCommand;

/// The process exit code returned by the Dust CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// The command succeeded.
    Success = 0,
    /// The command failed due to diagnostics or CLI misuse.
    Failure = 1,
    /// The `check` command found stale or missing outputs.
    Stale = 2,
}

impl ExitCode {
    /// Maps a command result into a process exit code.
    pub fn from_result(command: &CliCommand, result: &CommandResult) -> Self {
        if result.has_errors() {
            return Self::Failure;
        }

        if matches!(command, CliCommand::Check)
            && result.checked_libraries.iter().any(|library| library.stale)
        {
            return Self::Stale;
        }

        Self::Success
    }
}
