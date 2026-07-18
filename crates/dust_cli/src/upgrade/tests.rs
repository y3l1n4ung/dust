use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tempfile::TempDir;

use super::{
    UpgradeError,
    assets::{ArchiveKind, CHECKSUMS_FILE, current_release_asset, normalize_tag},
    flow::{UpgradeRequest, run_upgrade_with_io},
    report::{UpgradeMode, UpgradeReport, UpgradeStatus, render_upgrade_report},
    system::UpgradeIo,
};

/// In-memory upgrade IO used by unit tests.
#[derive(Debug)]
struct FakeUpgradeIo {
    /// Current executable returned to the workflow.
    current_exe: PathBuf,
    /// Latest release tag returned by metadata fetch.
    latest_tag: String,
    /// Bytes served for archive downloads.
    archive_bytes: Vec<u8>,
    /// Bytes used when generating the checksum manifest.
    checksum_bytes: Vec<u8>,
    /// Whether replacement should fail.
    fail_replace: bool,
    /// Captured latest metadata fetch count.
    latest_calls: Cell<usize>,
    /// Captured download URLs.
    downloads: RefCell<Vec<String>>,
    /// Captured replacement calls.
    replacements: Cell<usize>,
    /// Additional text responses by URL.
    text_by_url: HashMap<String, String>,
}

impl FakeUpgradeIo {
    /// Creates a fake IO boundary with a safe executable path.
    fn new(temp: &TempDir, archive_bytes: Vec<u8>) -> Self {
        let asset = current_release_asset().unwrap();
        let current_exe = temp.path().join("bin").join(asset.binary_name);
        fs::create_dir_all(current_exe.parent().unwrap()).unwrap();
        fs::write(&current_exe, b"current").unwrap();
        Self {
            current_exe,
            latest_tag: "v0.1.3".to_owned(),
            checksum_bytes: archive_bytes.clone(),
            archive_bytes,
            fail_replace: false,
            latest_calls: Cell::new(0),
            downloads: RefCell::new(Vec::new()),
            replacements: Cell::new(0),
            text_by_url: HashMap::new(),
        }
    }

    /// Builds a checksum manifest matching the fake archive bytes.
    fn checksum_manifest(&self) -> String {
        let asset = current_release_asset().unwrap();
        format!(
            "{}  {}\n",
            sha256_hex(&self.checksum_bytes),
            asset.asset_name
        )
    }
}

impl UpgradeIo for FakeUpgradeIo {
    fn current_exe(&self) -> Result<PathBuf, UpgradeError> {
        Ok(self.current_exe.clone())
    }

    fn fetch_text(&self, url: &str) -> Result<String, UpgradeError> {
        self.latest_calls.set(self.latest_calls.get() + 1);
        Ok(self
            .text_by_url
            .get(url)
            .cloned()
            .unwrap_or_else(|| format!("{{\"tag_name\":\"{}\"}}", self.latest_tag)))
    }

    fn download_to(&self, url: &str, destination: &Path) -> Result<(), UpgradeError> {
        self.downloads.borrow_mut().push(url.to_owned());
        if url.ends_with(CHECKSUMS_FILE) {
            fs::write(destination, self.checksum_manifest())
                .map_err(|source| UpgradeError::file("write", destination, source))?;
        } else {
            fs::write(destination, &self.archive_bytes)
                .map_err(|source| UpgradeError::file("write", destination, source))?;
        }
        Ok(())
    }

    fn extract_archive(
        &self,
        _archive: &Path,
        destination: &Path,
        _archive_kind: ArchiveKind,
        binary_name: &str,
    ) -> Result<PathBuf, UpgradeError> {
        let extracted = destination.join("release").join(binary_name);
        fs::create_dir_all(extracted.parent().unwrap())
            .map_err(|source| UpgradeError::file("create", extracted.parent().unwrap(), source))?;
        fs::write(&extracted, b"verified")
            .map_err(|source| UpgradeError::file("write", &extracted, source))?;
        Ok(extracted)
    }

    fn replace_binary(
        &self,
        _staged_binary: &Path,
        target_binary: &Path,
    ) -> Result<(), UpgradeError> {
        if self.fail_replace {
            return Err(UpgradeError::ReplacementFailed {
                target: target_binary.to_path_buf(),
                detail: "permission denied".to_owned(),
            });
        }
        self.replacements.set(self.replacements.get() + 1);
        Ok(())
    }
}

#[test]
fn check_reports_available_without_download() {
    let temp = TempDir::new().unwrap();
    let io = FakeUpgradeIo::new(&temp, b"archive".to_vec());
    let request = request(UpgradeMode::Check, None, "0.1.2");

    let report = run_upgrade_with_io(&request, &io).unwrap();

    assert_eq!(report.status, UpgradeStatus::Available);
    assert!(!report.checksum_verified);
    assert!(io.downloads.borrow().is_empty());
    assert_eq!(io.replacements.get(), 0);
}

#[test]
fn dry_run_verifies_without_replacing() {
    let temp = TempDir::new().unwrap();
    let io = FakeUpgradeIo::new(&temp, b"archive".to_vec());
    let request = request(UpgradeMode::DryRun, None, "0.1.2");

    let report = run_upgrade_with_io(&request, &io).unwrap();

    assert!(report.checksum_verified);
    assert!(!report.changed);
    assert_eq!(io.downloads.borrow().len(), 2);
    assert_eq!(io.replacements.get(), 0);
}

#[test]
fn apply_replaces_after_checksum_verification() {
    let temp = TempDir::new().unwrap();
    let io = FakeUpgradeIo::new(&temp, b"archive".to_vec());
    let request = request(UpgradeMode::Apply, None, "0.1.2");

    let report = run_upgrade_with_io(&request, &io).unwrap();

    assert!(report.checksum_verified);
    assert!(report.changed);
    assert_eq!(io.replacements.get(), 1);
}

#[test]
fn apply_skips_download_when_current_version_matches_target() {
    let temp = TempDir::new().unwrap();
    let mut io = FakeUpgradeIo::new(&temp, b"archive".to_vec());
    io.latest_tag = "v0.1.2".to_owned();
    let request = request(UpgradeMode::Apply, None, "0.1.2");

    let report = run_upgrade_with_io(&request, &io).unwrap();

    assert_eq!(report.status, UpgradeStatus::UpToDate);
    assert!(io.downloads.borrow().is_empty());
    assert_eq!(io.replacements.get(), 0);
}

#[test]
fn checksum_mismatch_does_not_replace() {
    let temp = TempDir::new().unwrap();
    let mut io = FakeUpgradeIo::new(&temp, b"archive".to_vec());
    io.archive_bytes = b"changed archive".to_vec();
    let request = request(UpgradeMode::DryRun, Some("v0.1.3"), "0.1.2");

    let error = run_upgrade_with_io(&request, &io).unwrap_err();

    assert!(error.to_string().contains("checksum mismatch"));
    assert_eq!(io.replacements.get(), 0);
}

#[test]
fn replacement_failure_reports_without_changed_flag() {
    let temp = TempDir::new().unwrap();
    let mut io = FakeUpgradeIo::new(&temp, b"archive".to_vec());
    io.fail_replace = true;
    let request = request(UpgradeMode::Apply, Some("v0.1.3"), "0.1.2");

    let error = run_upgrade_with_io(&request, &io).unwrap_err();

    assert!(error.to_string().contains("could not replace"));
    assert_eq!(io.replacements.get(), 0);
}

#[test]
fn explicit_tag_avoids_latest_metadata_fetch() {
    let temp = TempDir::new().unwrap();
    let io = FakeUpgradeIo::new(&temp, b"archive".to_vec());
    let request = request(UpgradeMode::DryRun, Some("0.1.3"), "0.1.2");

    let report = run_upgrade_with_io(&request, &io).unwrap();

    assert_eq!(report.target_tag, "v0.1.3");
    assert_eq!(io.latest_calls.get(), 0);
}

#[test]
fn refuses_development_binary_for_apply() {
    let temp = TempDir::new().unwrap();
    let asset = current_release_asset().unwrap();
    let target = temp.path().join("target/debug").join(asset.binary_name);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, b"dev").unwrap();
    let mut io = FakeUpgradeIo::new(&temp, b"archive".to_vec());
    io.current_exe = target;
    let request = request(UpgradeMode::Apply, Some("v0.1.3"), "0.1.2");

    let error = run_upgrade_with_io(&request, &io).unwrap_err();

    assert!(error.to_string().contains("refusing to replace"));
    assert!(io.downloads.borrow().is_empty());
}

#[test]
fn renders_deterministic_upgrade_report() {
    let report = UpgradeReport {
        mode: UpgradeMode::Check,
        status: UpgradeStatus::Available,
        current_tag: "v0.1.2".to_owned(),
        target_tag: "v0.1.3".to_owned(),
        asset_name: "dust-x86_64-unknown-linux-gnu.tar.gz".to_owned(),
        checksum_verified: false,
        binary_path: PathBuf::from("/usr/local/bin/dust"),
        changed: false,
        elapsed_ms: 7,
    };

    let rendered = render_upgrade_report(&report);

    assert!(rendered.contains("mode: check"));
    assert!(rendered.contains("status: available"));
    assert!(rendered.contains("checksum  skipped"));
    assert!(rendered.ends_with("changed  false\n"));
}

/// Builds an internal upgrade request.
fn request(mode: UpgradeMode, tag: Option<&str>, current_version: &str) -> UpgradeRequest {
    UpgradeRequest {
        mode,
        tag: tag.map(normalize_tag).transpose().unwrap(),
        current_version: current_version.to_owned(),
    }
}

/// Returns a lowercase SHA-256 digest for test bytes.
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
