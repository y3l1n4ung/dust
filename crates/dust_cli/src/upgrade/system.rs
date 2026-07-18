use std::{
    env, fs,
    path::{Path, PathBuf},
};

use super::{
    UpgradeError,
    assets::ArchiveKind,
    commands::{
        download_program_name, extract_program_name, require_success, trusted_download_command,
        trusted_extract_command, trusted_fetch_command,
    },
};

/// IO boundary used by the upgrade workflow.
pub(crate) trait UpgradeIo {
    /// Resolves the currently running executable path.
    fn current_exe(&self) -> Result<PathBuf, UpgradeError>;

    /// Fetches a trusted text URL.
    fn fetch_text(&self, url: &str) -> Result<String, UpgradeError>;

    /// Downloads a trusted URL to the provided destination path.
    fn download_to(&self, url: &str, destination: &Path) -> Result<(), UpgradeError>;

    /// Extracts a release archive and returns the extracted binary path.
    fn extract_archive(
        &self,
        archive: &Path,
        destination: &Path,
        archive_kind: ArchiveKind,
        binary_name: &str,
    ) -> Result<PathBuf, UpgradeError>;

    /// Replaces the current executable with a verified staged binary.
    fn replace_binary(
        &self,
        staged_binary: &Path,
        target_binary: &Path,
    ) -> Result<(), UpgradeError>;
}

/// Production implementation of the upgrade IO boundary.
#[derive(Debug, Default)]
pub(crate) struct SystemUpgradeIo;

impl UpgradeIo for SystemUpgradeIo {
    fn current_exe(&self) -> Result<PathBuf, UpgradeError> {
        env::current_exe()
            .map_err(|source| UpgradeError::CurrentExecutable(source.to_string()))
            .and_then(|path| {
                if path.is_absolute() {
                    Ok(path)
                } else {
                    Err(UpgradeError::UnsafeExecutablePath(path))
                }
            })
    }

    fn fetch_text(&self, url: &str) -> Result<String, UpgradeError> {
        let output =
            trusted_fetch_command(url)
                .output()
                .map_err(|source| UpgradeError::CommandFailed {
                    program: download_program_name().to_owned(),
                    detail: source.to_string(),
                })?;
        require_success(download_program_name(), output).and_then(|bytes| {
            String::from_utf8(bytes).map_err(|source| UpgradeError::InvalidResponse {
                url: url.to_owned(),
                detail: source.to_string(),
            })
        })
    }

    fn download_to(&self, url: &str, destination: &Path) -> Result<(), UpgradeError> {
        let output = trusted_download_command(url, destination)
            .output()
            .map_err(|source| UpgradeError::CommandFailed {
                program: download_program_name().to_owned(),
                detail: source.to_string(),
            })?;
        require_success(download_program_name(), output).map(|_| ())
    }

    fn extract_archive(
        &self,
        archive: &Path,
        destination: &Path,
        archive_kind: ArchiveKind,
        binary_name: &str,
    ) -> Result<PathBuf, UpgradeError> {
        let output = trusted_extract_command(archive, destination, archive_kind)
            .output()
            .map_err(|source| UpgradeError::CommandFailed {
                program: extract_program_name(archive_kind).to_owned(),
                detail: source.to_string(),
            })?;
        require_success(extract_program_name(archive_kind), output)?;
        find_extracted_binary(destination, binary_name)
    }

    fn replace_binary(
        &self,
        staged_binary: &Path,
        target_binary: &Path,
    ) -> Result<(), UpgradeError> {
        let parent = target_binary
            .parent()
            .ok_or_else(|| UpgradeError::UnsafeExecutablePath(target_binary.to_path_buf()))?;
        let temporary_target = parent.join(format!(".dust-upgrade-{}.tmp", std::process::id()));
        fs::copy(staged_binary, &temporary_target)
            .map_err(|source| UpgradeError::file("stage", &temporary_target, source))?;
        preserve_executable_permissions(target_binary, &temporary_target)?;

        match fs::rename(&temporary_target, target_binary) {
            Ok(()) => Ok(()),
            Err(source) => {
                let _ = fs::remove_file(&temporary_target);
                Err(UpgradeError::ReplacementFailed {
                    target: target_binary.to_path_buf(),
                    detail: source.to_string(),
                })
            }
        }
    }
}

/// Finds an extracted binary under the archive extraction root.
fn find_extracted_binary(root: &Path, binary_name: &str) -> Result<PathBuf, UpgradeError> {
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in
            fs::read_dir(&path).map_err(|source| UpgradeError::file("read", &path, source))?
        {
            let entry = entry.map_err(|source| UpgradeError::file("read", &path, source))?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                pending.push(entry_path);
            } else if entry_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == binary_name)
            {
                return Ok(entry_path);
            }
        }
    }

    Err(UpgradeError::ExtractedBinaryMissing {
        binary: binary_name.to_owned(),
        root: root.to_path_buf(),
    })
}

/// Preserves executable permissions from the current binary on Unix.
#[cfg(unix)]
fn preserve_executable_permissions(source: &Path, destination: &Path) -> Result<(), UpgradeError> {
    use std::os::unix::fs::PermissionsExt;

    let source_permissions = fs::metadata(source)
        .map_err(|source_error| UpgradeError::file("read", source, source_error))?
        .permissions()
        .mode();
    let mut destination_permissions = fs::metadata(destination)
        .map_err(|source_error| UpgradeError::file("read", destination, source_error))?
        .permissions();
    destination_permissions.set_mode(source_permissions | 0o111);
    fs::set_permissions(destination, destination_permissions)
        .map_err(|source_error| UpgradeError::file("chmod", destination, source_error))
}

/// Preserves executable permissions from the current binary on non-Unix platforms.
#[cfg(not(unix))]
fn preserve_executable_permissions(
    _source: &Path,
    _destination: &Path,
) -> Result<(), UpgradeError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn finds_extracted_binary_recursively() {
        let temp = TempDir::new().unwrap();
        let nested = temp.path().join("dust-release/bin");
        fs::create_dir_all(&nested).unwrap();
        let binary = nested.join("dust");
        fs::write(&binary, b"binary").unwrap();

        assert_eq!(find_extracted_binary(temp.path(), "dust").unwrap(), binary);
    }
}
