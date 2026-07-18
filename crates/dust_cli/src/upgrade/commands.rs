use std::{
    path::Path,
    process::{Command, Output},
};

use super::{UpgradeError, assets::ArchiveKind};

/// Builds the platform-specific trusted text fetch command.
#[cfg(not(windows))]
pub(super) fn trusted_fetch_command(url: &str) -> Command {
    let mut command = Command::new("curl");
    command.args(["-fsSL", "--proto", "=https", "--tlsv1.2", url]);
    command
}

/// Builds the platform-specific trusted text fetch command.
#[cfg(windows)]
pub(super) fn trusted_fetch_command(url: &str) -> Command {
    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        "$ProgressPreference = 'SilentlyContinue'; \
         (Invoke-WebRequest -UseBasicParsing -Uri $args[0]).Content",
    ]);
    command.arg(url);
    command
}

/// Builds the platform-specific trusted download command.
#[cfg(not(windows))]
pub(super) fn trusted_download_command(url: &str, destination: &Path) -> Command {
    let mut command = Command::new("curl");
    command.args(["-fsSL", "--proto", "=https", "--tlsv1.2", "-o"]);
    command.arg(destination).arg(url);
    command
}

/// Builds the platform-specific trusted download command.
#[cfg(windows)]
pub(super) fn trusted_download_command(url: &str, destination: &Path) -> Command {
    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        "$ProgressPreference = 'SilentlyContinue'; \
         Invoke-WebRequest -UseBasicParsing -Uri $args[0] -OutFile $args[1]",
    ]);
    command.arg(url).arg(destination);
    command
}

/// Builds the platform-specific archive extraction command.
#[cfg(not(windows))]
pub(super) fn trusted_extract_command(
    archive: &Path,
    destination: &Path,
    archive_kind: ArchiveKind,
) -> Command {
    match archive_kind {
        ArchiveKind::TarGz => {
            let mut command = Command::new("tar");
            command.arg("-xzf").arg(archive).arg("-C").arg(destination);
            command
        }
        ArchiveKind::Zip => {
            let mut command = Command::new("unzip");
            command.arg("-q").arg(archive).arg("-d").arg(destination);
            command
        }
    }
}

/// Builds the platform-specific archive extraction command.
#[cfg(windows)]
pub(super) fn trusted_extract_command(
    archive: &Path,
    destination: &Path,
    archive_kind: ArchiveKind,
) -> Command {
    match archive_kind {
        ArchiveKind::TarGz => {
            let mut command = Command::new("tar.exe");
            command.arg("-xzf").arg(archive).arg("-C").arg(destination);
            command
        }
        ArchiveKind::Zip => {
            let mut command = Command::new("powershell.exe");
            command.args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "Expand-Archive -Force -Path $args[0] -DestinationPath $args[1]",
            ]);
            command.arg(archive).arg(destination);
            command
        }
    }
}

/// Returns the configured download program name for diagnostics.
#[cfg(not(windows))]
pub(super) fn download_program_name() -> &'static str {
    "curl"
}

/// Returns the configured download program name for diagnostics.
#[cfg(windows)]
pub(super) fn download_program_name() -> &'static str {
    "powershell.exe"
}

/// Returns the configured extraction program name for diagnostics.
#[cfg(not(windows))]
pub(super) fn extract_program_name(archive_kind: ArchiveKind) -> &'static str {
    match archive_kind {
        ArchiveKind::TarGz => "tar",
        ArchiveKind::Zip => "unzip",
    }
}

/// Returns the configured extraction program name for diagnostics.
#[cfg(windows)]
pub(super) fn extract_program_name(archive_kind: ArchiveKind) -> &'static str {
    match archive_kind {
        ArchiveKind::TarGz => "tar.exe",
        ArchiveKind::Zip => "powershell.exe",
    }
}

/// Converts a command output into stdout bytes or a user-facing error.
pub(super) fn require_success(program: &str, output: Output) -> Result<Vec<u8>, UpgradeError> {
    if output.status.success() {
        return Ok(output.stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let detail = if stderr.is_empty() {
        format!("exited with status {}", output.status)
    } else {
        stderr
    };
    Err(UpgradeError::CommandFailed {
        program: program.to_owned(),
        detail,
    })
}
