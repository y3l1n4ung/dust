use std::path::{Path, PathBuf};

use dust_diagnostics::Diagnostic;

/// Detects the nearest Dart package root by walking upward from the given path.
///
/// A directory is considered a package root when it contains at least one of:
/// - `pubspec.yaml`
/// - `dust.yaml`
pub fn detect_workspace_root(cwd: &Path) -> Result<PathBuf, Diagnostic> {
    let cwd = std::path::absolute(cwd).map_err(|error| {
        Diagnostic::error(format!(
            "failed to resolve workspace path `{}`: {error}",
            cwd.display()
        ))
    })?;
    let mut current = if cwd.is_dir() {
        cwd
    } else {
        cwd.parent()
            .ok_or_else(|| {
                Diagnostic::error("cannot determine parent directory for workspace discovery")
            })?
            .to_path_buf()
    };

    loop {
        if current.join("pubspec.yaml").is_file() || current.join("dust.yaml").is_file() {
            return Ok(current);
        }

        if !current.pop() {
            return Err(Diagnostic::error("no Dart package root found"));
        }
    }
}
