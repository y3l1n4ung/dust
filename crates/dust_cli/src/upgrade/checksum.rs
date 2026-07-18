use std::{fs::File, io::Read, path::Path};

use sha2::{Digest, Sha256};

use super::UpgradeError;

/// Successful checksum verification details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChecksumVerification {
    /// Expected checksum read from the release manifest.
    pub(crate) expected: String,
    /// Actual checksum calculated from the downloaded archive.
    pub(crate) actual: String,
}

/// Verifies a downloaded release archive against the checksum manifest.
pub(crate) fn verify_archive_checksum(
    checksums: &str,
    asset_name: &str,
    archive_path: &Path,
) -> Result<ChecksumVerification, UpgradeError> {
    let expected = expected_checksum(checksums, asset_name)?;
    let actual = sha256_file(archive_path)?;
    if expected.eq_ignore_ascii_case(&actual) {
        return Ok(ChecksumVerification { expected, actual });
    }

    Err(UpgradeError::ChecksumMismatch {
        asset: asset_name.to_owned(),
        expected,
        actual,
    })
}

/// Reads the expected checksum for one asset from a sha256sum-style manifest.
fn expected_checksum(checksums: &str, asset_name: &str) -> Result<String, UpgradeError> {
    for line in checksums
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let mut parts = line.split_whitespace();
        let Some(hash) = parts.next() else {
            continue;
        };
        let Some(path) = parts.next() else {
            continue;
        };
        if checksum_path_matches(path, asset_name) {
            return Ok(hash.to_ascii_lowercase());
        }
    }

    Err(UpgradeError::MissingChecksum {
        asset: asset_name.to_owned(),
    })
}

/// Returns whether a checksum manifest path names the selected asset.
fn checksum_path_matches(path: &str, asset_name: &str) -> bool {
    let normalized = path.trim_start_matches('*');
    normalized == asset_name
        || Path::new(normalized)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            == Some(asset_name)
}

/// Computes the SHA-256 checksum of a file as lowercase hex.
fn sha256_file(path: &Path) -> Result<String, UpgradeError> {
    let mut file = File::open(path).map_err(|source| UpgradeError::file("read", path, source))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|source| UpgradeError::file("read", path, source))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Convenience helper for test fixtures that need real archive hashes.
#[cfg(test)]
fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn reads_checksum_by_filename() {
        let hash = "A7507D819769D01A365AB707794A4084392C824F54A7A6A7862F8C3D0892B283";
        let checksums = format!("{hash}  ./dist/dust-aarch64-apple-darwin.tar.gz\n");

        assert_eq!(
            expected_checksum(&checksums, "dust-aarch64-apple-darwin.tar.gz").unwrap(),
            hash.to_ascii_lowercase()
        );
    }

    #[test]
    fn verifies_archive_hash() {
        let archive = NamedTempFile::new().unwrap();
        std::fs::write(archive.path(), b"dust archive").unwrap();
        let hash = sha256_bytes(b"dust archive");
        let checksums = format!("{hash}  dust-x86_64-unknown-linux-gnu.tar.gz\n");

        let verification = verify_archive_checksum(
            &checksums,
            "dust-x86_64-unknown-linux-gnu.tar.gz",
            archive.path(),
        )
        .unwrap();

        assert_eq!(verification.expected, hash);
        assert_eq!(verification.actual, hash);
    }

    #[test]
    fn reports_checksum_mismatch() {
        let archive = NamedTempFile::new().unwrap();
        std::fs::write(archive.path(), b"dust archive").unwrap();
        let checksums = "0000  dust-x86_64-unknown-linux-gnu.tar.gz\n";

        let error = verify_archive_checksum(
            checksums,
            "dust-x86_64-unknown-linux-gnu.tar.gz",
            archive.path(),
        )
        .unwrap_err();

        assert!(error.to_string().contains("checksum mismatch"));
    }
}
