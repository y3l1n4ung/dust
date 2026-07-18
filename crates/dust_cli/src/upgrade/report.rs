use std::{fmt, path::PathBuf};

use super::UpgradeError;

/// Rendered CLI output from the upgrade command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UpgradeCliOutput {
    /// Process exit code.
    pub(crate) exit_code: i32,
    /// Text written to standard output.
    pub(crate) stdout: String,
    /// Text written to standard error.
    pub(crate) stderr: String,
}

impl UpgradeCliOutput {
    /// Builds successful CLI output.
    pub(super) fn success(stdout: String) -> Self {
        Self {
            exit_code: 0,
            stdout,
            stderr: String::new(),
        }
    }

    /// Builds failed CLI output.
    pub(super) fn failure(error: UpgradeError) -> Self {
        Self {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("dust upgrade failed: {error}\n"),
        }
    }
}

/// User-selected upgrade mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UpgradeMode {
    /// Check release metadata only.
    Check,
    /// Download and verify release assets without replacement.
    DryRun,
    /// Download, verify, and replace the installed binary.
    Apply,
}

impl fmt::Display for UpgradeMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Check => formatter.write_str("check"),
            Self::DryRun => formatter.write_str("dry-run"),
            Self::Apply => formatter.write_str("apply"),
        }
    }
}

/// Whether the selected release differs from the current CLI version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UpgradeStatus {
    /// Current binary already matches the selected target tag.
    UpToDate,
    /// Selected target tag differs from the current binary version.
    Available,
}

impl fmt::Display for UpgradeStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpToDate => formatter.write_str("up-to-date"),
            Self::Available => formatter.write_str("available"),
        }
    }
}

/// Successful upgrade command report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UpgradeReport {
    /// Selected command mode.
    pub(super) mode: UpgradeMode,
    /// Whether a different release was selected.
    pub(super) status: UpgradeStatus,
    /// Current CLI version with a `v` prefix.
    pub(super) current_tag: String,
    /// Selected target release tag.
    pub(super) target_tag: String,
    /// Selected release asset filename.
    pub(super) asset_name: String,
    /// Whether archive checksum verification completed.
    pub(super) checksum_verified: bool,
    /// Path of the executable considered by the command.
    pub(super) binary_path: PathBuf,
    /// Whether the installed binary was replaced.
    pub(super) changed: bool,
    /// Elapsed command time in milliseconds.
    pub(super) elapsed_ms: u128,
}

/// Builds the current version tag.
pub(super) fn current_tag(current_version: &str) -> String {
    format!("v{current_version}")
}

/// Compares the selected release tag with the current package version.
pub(super) fn release_status(current_version: &str, target_tag: &str) -> UpgradeStatus {
    if target_tag.trim_start_matches('v') == current_version {
        UpgradeStatus::UpToDate
    } else {
        UpgradeStatus::Available
    }
}

/// Renders an upgrade report as deterministic CLI output.
pub(super) fn render_upgrade_report(report: &UpgradeReport) -> String {
    let checksum = if report.checksum_verified {
        "verified"
    } else {
        "skipped"
    };

    format!(
        "dust upgrade  mode: {}  status: {}  current: {}  target: {}  time: {}ms\nasset  {}\nchecksum  {checksum}\npath  {}\nchanged  {}\n",
        report.mode,
        report.status,
        report.current_tag,
        report.target_tag,
        report.elapsed_ms,
        report.asset_name,
        report.binary_path.display(),
        report.changed
    )
}
