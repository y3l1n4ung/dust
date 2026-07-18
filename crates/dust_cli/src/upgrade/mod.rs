//! Binary upgrade command implementation.

/// Release asset mapping and URL construction.
mod assets;
/// Release checksum parsing and verification.
mod checksum;
/// Platform command builders for trusted network and archive operations.
mod commands;
/// Upgrade error reporting.
mod error;
/// Upgrade workflow orchestration.
mod flow;
/// Upgrade report rendering.
mod report;
/// Production system IO for downloads, extraction, and replacement.
mod system;
/// Upgrade workflow tests with fake IO.
#[cfg(test)]
mod tests;

pub(crate) use error::UpgradeError;
pub(crate) use flow::run_upgrade;
