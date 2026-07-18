use std::{fs, path::Path, time::Instant};

use serde_json::Value;
use tempfile::Builder;

use crate::args::CliOptions;

use super::{
    UpgradeError,
    assets::{
        ArchiveKind, CHECKSUMS_FILE, RELEASE_REPOSITORY, current_release_asset,
        latest_release_api_url, normalize_tag, release_download_url,
    },
    checksum::verify_archive_checksum,
    report::{
        UpgradeCliOutput, UpgradeMode, UpgradeReport, UpgradeStatus, current_tag, release_status,
        render_upgrade_report,
    },
    system::{SystemUpgradeIo, UpgradeIo},
};

/// Internal request used by the upgrade workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UpgradeRequest {
    /// Selected command mode.
    pub(super) mode: UpgradeMode,
    /// Optional explicit release tag.
    pub(super) tag: Option<String>,
    /// Current CLI version without the `v` prefix.
    pub(super) current_version: String,
}

/// Runs the upgrade command against production IO.
pub(crate) fn run_upgrade(options: &CliOptions) -> UpgradeCliOutput {
    let request = match upgrade_request_from_options(options) {
        Ok(request) => request,
        Err(error) => return UpgradeCliOutput::failure(error),
    };
    let io = SystemUpgradeIo;
    match run_upgrade_with_io(&request, &io) {
        Ok(report) => UpgradeCliOutput::success(render_upgrade_report(&report)),
        Err(error) => UpgradeCliOutput::failure(error),
    }
}

/// Converts parsed CLI options into an upgrade request.
fn upgrade_request_from_options(options: &CliOptions) -> Result<UpgradeRequest, UpgradeError> {
    let mode = match (options.upgrade_check, options.upgrade_dry_run) {
        (true, false) => UpgradeMode::Check,
        (false, true) => UpgradeMode::DryRun,
        (false, false) => UpgradeMode::Apply,
        (true, true) => UpgradeMode::Check,
    };
    let tag = options
        .upgrade_tag
        .as_deref()
        .map(normalize_tag)
        .transpose()?;

    Ok(UpgradeRequest {
        mode,
        tag,
        current_version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

/// Runs the upgrade workflow against an injected IO boundary.
pub(super) fn run_upgrade_with_io(
    request: &UpgradeRequest,
    io: &dyn UpgradeIo,
) -> Result<UpgradeReport, UpgradeError> {
    let started = Instant::now();
    let asset = current_release_asset()?;
    let binary_path = io.current_exe()?;
    let target_tag = resolve_target_tag(request, io)?;
    let current_tag = current_tag(&request.current_version);
    let status = release_status(&request.current_version, &target_tag);

    if request.mode == UpgradeMode::Apply {
        validate_replacement_target(&binary_path, asset.binary_name)?;
    }

    let mut report = UpgradeReport {
        mode: request.mode,
        status,
        current_tag,
        target_tag,
        asset_name: asset.asset_name.to_owned(),
        checksum_verified: false,
        binary_path,
        changed: false,
        elapsed_ms: 0,
    };

    if request.mode == UpgradeMode::Check
        || (request.mode == UpgradeMode::Apply && status == UpgradeStatus::UpToDate)
    {
        report.elapsed_ms = started.elapsed().as_millis();
        return Ok(report);
    }

    verify_release_asset(io, &mut report, asset.archive_kind, asset.binary_name)?;

    report.elapsed_ms = started.elapsed().as_millis();
    Ok(report)
}

/// Resolves the release tag from either CLI input or GitHub latest metadata.
fn resolve_target_tag(
    request: &UpgradeRequest,
    io: &dyn UpgradeIo,
) -> Result<String, UpgradeError> {
    if let Some(tag) = &request.tag {
        return Ok(tag.clone());
    }

    let url = latest_release_api_url(RELEASE_REPOSITORY);
    let body = io.fetch_text(&url)?;
    let json: Value =
        serde_json::from_str(&body).map_err(|source| UpgradeError::InvalidResponse {
            url: url.clone(),
            detail: source.to_string(),
        })?;
    json.get("tag_name")
        .and_then(Value::as_str)
        .ok_or_else(|| UpgradeError::InvalidResponse {
            url,
            detail: "missing tag_name".to_owned(),
        })
        .and_then(normalize_tag)
}

/// Downloads and verifies the selected archive.
fn verify_release_asset(
    io: &dyn UpgradeIo,
    report: &mut UpgradeReport,
    archive_kind: ArchiveKind,
    binary_name: &str,
) -> Result<(), UpgradeError> {
    let temp = Builder::new()
        .prefix("dust-upgrade-")
        .tempdir()
        .map_err(|source| UpgradeError::file("create", Path::new("tempdir"), source))?;
    let checksums_path = temp.path().join(CHECKSUMS_FILE);
    let archive_path = temp.path().join(&report.asset_name);

    let checksums_url =
        release_download_url(RELEASE_REPOSITORY, &report.target_tag, CHECKSUMS_FILE);
    let archive_url =
        release_download_url(RELEASE_REPOSITORY, &report.target_tag, &report.asset_name);
    io.download_to(&checksums_url, &checksums_path)?;
    io.download_to(&archive_url, &archive_path)?;

    let checksums = fs::read_to_string(&checksums_path)
        .map_err(|source| UpgradeError::file("read", &checksums_path, source))?;
    verify_archive_checksum(&checksums, &report.asset_name, &archive_path)?;
    report.checksum_verified = true;

    if report.mode == UpgradeMode::Apply {
        let extracted =
            io.extract_archive(&archive_path, temp.path(), archive_kind, binary_name)?;
        io.replace_binary(&extracted, &report.binary_path)?;
        report.changed = true;
    }

    Ok(())
}

/// Refuses replacement of development builds and unsafe executable names.
fn validate_replacement_target(
    path: &Path,
    expected_binary_name: &str,
) -> Result<(), UpgradeError> {
    if !path.is_absolute()
        || path.file_name().and_then(|name| name.to_str()) != Some(expected_binary_name)
    {
        return Err(UpgradeError::UnsafeExecutablePath(path.to_path_buf()));
    }

    let components = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>();
    if components
        .windows(2)
        .any(|window| window[0] == "target" && matches!(window[1].as_ref(), "debug" | "release"))
    {
        return Err(UpgradeError::UnsafeExecutablePath(path.to_path_buf()));
    }

    Ok(())
}
