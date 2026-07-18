use std::env;

use super::UpgradeError;

/// GitHub repository that owns trusted Dust release artifacts.
pub(crate) const RELEASE_REPOSITORY: &str = "y3l1n4ung/dust";

/// Release checksum manifest filename.
pub(crate) const CHECKSUMS_FILE: &str = "SHA256SUMS.txt";

/// Archive format used by a release asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArchiveKind {
    /// Unix-style gzip-compressed tar archive.
    TarGz,
    /// Windows zip archive.
    Zip,
}

/// Release asset selected for the current platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReleaseAsset {
    /// Asset filename published by the release workflow.
    pub(crate) asset_name: &'static str,
    /// Executable filename expected inside the archive.
    pub(crate) binary_name: &'static str,
    /// Archive format used by the asset.
    pub(crate) archive_kind: ArchiveKind,
}

/// Returns the release asset for the host platform.
pub(crate) fn current_release_asset() -> Result<ReleaseAsset, UpgradeError> {
    release_asset_for(env::consts::OS, env::consts::ARCH)
}

/// Maps Rust target OS and architecture strings to release assets.
pub(crate) fn release_asset_for(os: &str, arch: &str) -> Result<ReleaseAsset, UpgradeError> {
    match (os, arch) {
        ("macos", "aarch64") => Ok(ReleaseAsset {
            asset_name: "dust-aarch64-apple-darwin.tar.gz",
            binary_name: "dust",
            archive_kind: ArchiveKind::TarGz,
        }),
        ("macos", "x86_64") => Ok(ReleaseAsset {
            asset_name: "dust-x86_64-apple-darwin.tar.gz",
            binary_name: "dust",
            archive_kind: ArchiveKind::TarGz,
        }),
        ("linux", "x86_64") => Ok(ReleaseAsset {
            asset_name: "dust-x86_64-unknown-linux-gnu.tar.gz",
            binary_name: "dust",
            archive_kind: ArchiveKind::TarGz,
        }),
        ("windows", "x86_64") => Ok(ReleaseAsset {
            asset_name: "dust-x86_64-pc-windows-msvc.zip",
            binary_name: "dust.exe",
            archive_kind: ArchiveKind::Zip,
        }),
        _ => Err(UpgradeError::UnsupportedPlatform {
            os: os.to_owned(),
            arch: arch.to_owned(),
        }),
    }
}

/// Builds the GitHub API URL for latest release metadata.
pub(crate) fn latest_release_api_url(repository: &str) -> String {
    format!("https://api.github.com/repos/{repository}/releases/latest")
}

/// Builds the GitHub release asset download URL.
pub(crate) fn release_download_url(repository: &str, tag: &str, filename: &str) -> String {
    format!("https://github.com/{repository}/releases/download/{tag}/{filename}")
}

/// Normalizes user-provided release tags while keeping URL path input strict.
pub(crate) fn normalize_tag(tag: &str) -> Result<String, UpgradeError> {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        return Err(UpgradeError::InvalidTag(tag.to_owned()));
    }

    let normalized = if trimmed.starts_with('v') {
        trimmed.to_owned()
    } else {
        format!("v{trimmed}")
    };
    if normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
    {
        Ok(normalized)
    } else {
        Err(UpgradeError::InvalidTag(tag.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_installer_release_assets() {
        assert_eq!(
            release_asset_for("macos", "aarch64").unwrap().asset_name,
            "dust-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            release_asset_for("macos", "x86_64").unwrap().asset_name,
            "dust-x86_64-apple-darwin.tar.gz"
        );
        assert_eq!(
            release_asset_for("linux", "x86_64").unwrap().asset_name,
            "dust-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            release_asset_for("windows", "x86_64").unwrap().asset_name,
            "dust-x86_64-pc-windows-msvc.zip"
        );
    }

    #[test]
    fn rejects_unsupported_release_assets() {
        let error = release_asset_for("linux", "aarch64").unwrap_err();

        assert!(error.to_string().contains("unsupported platform"));
    }

    #[test]
    fn normalizes_release_tags() {
        assert_eq!(normalize_tag("0.1.3").unwrap(), "v0.1.3");
        assert_eq!(normalize_tag("v0.1.3").unwrap(), "v0.1.3");
        assert!(normalize_tag("../v0.1.3").is_err());
    }
}
