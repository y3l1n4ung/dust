use std::{
    fmt, io,
    path::{Path, PathBuf},
};

/// Upgrade command failure.
#[derive(Debug)]
pub(crate) enum UpgradeError {
    /// Current platform has no release artifact in the installer matrix.
    UnsupportedPlatform {
        /// Rust target operating system string.
        os: String,
        /// Rust target architecture string.
        arch: String,
    },
    /// User-provided release tag is empty or unsafe for a GitHub release URL.
    InvalidTag(String),
    /// The running executable path could not be resolved.
    CurrentExecutable(String),
    /// The executable path is unsafe to replace.
    UnsafeExecutablePath(PathBuf),
    /// A required system command failed.
    CommandFailed {
        /// Program that failed.
        program: String,
        /// Failure detail.
        detail: String,
    },
    /// GitHub response could not be parsed or was missing required data.
    InvalidResponse {
        /// URL that produced the invalid response.
        url: String,
        /// Failure detail.
        detail: String,
    },
    /// A filesystem operation failed.
    File {
        /// Operation being attempted.
        action: &'static str,
        /// Path being accessed.
        path: PathBuf,
        /// Failure detail.
        detail: String,
    },
    /// Release checksum manifest does not include the selected asset.
    MissingChecksum {
        /// Asset filename missing from the manifest.
        asset: String,
    },
    /// Downloaded archive checksum did not match the release manifest.
    ChecksumMismatch {
        /// Asset filename that failed verification.
        asset: String,
        /// Expected checksum from the manifest.
        expected: String,
        /// Actual checksum calculated from the downloaded file.
        actual: String,
    },
    /// Archive extraction succeeded but no Dust binary was found.
    ExtractedBinaryMissing {
        /// Expected binary filename.
        binary: String,
        /// Extraction root searched for the binary.
        root: PathBuf,
    },
    /// Verified staged binary could not replace the target executable.
    ReplacementFailed {
        /// Target executable path.
        target: PathBuf,
        /// Failure detail.
        detail: String,
    },
}

impl UpgradeError {
    /// Builds a filesystem error.
    pub(crate) fn file(action: &'static str, path: &Path, source: io::Error) -> Self {
        Self::File {
            action,
            path: path.to_path_buf(),
            detail: source.to_string(),
        }
    }
}

impl fmt::Display for UpgradeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform { os, arch } => write!(
                formatter,
                "unsupported platform {os}/{arch}; no Dust release asset is available"
            ),
            Self::InvalidTag(tag) => write!(
                formatter,
                "invalid release tag {tag:?}; use a tag like v0.1.3"
            ),
            Self::CurrentExecutable(detail) => {
                write!(formatter, "could not resolve current executable: {detail}")
            }
            Self::UnsafeExecutablePath(path) => write!(
                formatter,
                "refusing to replace unsafe executable path {}; install Dust with the release installer first",
                path.display()
            ),
            Self::CommandFailed { program, detail } => {
                write!(formatter, "{program} failed: {detail}")
            }
            Self::InvalidResponse { url, detail } => {
                write!(formatter, "invalid response from {url}: {detail}")
            }
            Self::File {
                action,
                path,
                detail,
            } => write!(formatter, "could not {action} {}: {detail}", path.display()),
            Self::MissingChecksum { asset } => {
                write!(
                    formatter,
                    "checksum for {asset} was not found in SHA256SUMS.txt"
                )
            }
            Self::ChecksumMismatch {
                asset,
                expected,
                actual,
            } => write!(
                formatter,
                "checksum mismatch for {asset}; expected {expected}, got {actual}"
            ),
            Self::ExtractedBinaryMissing { binary, root } => write!(
                formatter,
                "archive did not contain {binary} under {}",
                root.display()
            ),
            Self::ReplacementFailed { target, detail } => {
                write!(
                    formatter,
                    "could not replace {}: {detail}",
                    target.display()
                )
            }
        }
    }
}
