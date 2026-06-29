use std::{
    fs, io,
    path::Path,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

/// Writes one file atomically within its parent directory.
pub(super) fn write_atomic(path: &Path, source: &str) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output path has no parent"))?;
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "output path has no file name")
        })?;
    let temp_path = parent.join(format!(
        ".{file_name}.{}.{}.tmp",
        process::id(),
        unique_token()
    ));
    fs::write(&temp_path, source)?;
    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }
    Ok(())
}

/// Returns a best-effort unique token for temporary file names.
fn unique_token() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}
