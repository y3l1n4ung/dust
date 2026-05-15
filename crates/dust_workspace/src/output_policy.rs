use std::path::{Path, PathBuf};

use dust_diagnostics::Diagnostic;

/// Returns the configured primary generated output path for one source library.
pub fn primary_output_path(source_path: &Path, primary_suffix: &str) -> PathBuf {
    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("file");
    source_path.with_file_name(format!("{stem}{primary_suffix}"))
}

/// Returns the generated part URI that the source library must declare.
pub fn expected_part_uri(source_path: &Path, primary_suffix: &str) -> String {
    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("file");
    format!("{stem}{primary_suffix}")
}

/// Returns `true` when the path already points at a generated primary output.
pub fn is_generated_primary_file(path: &Path, primary_suffix: &str) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(primary_suffix))
}

/// Returns the shared generated test output path for one source library.
pub fn generated_test_output_path(
    package_root: &Path,
    source_path: &Path,
) -> Result<PathBuf, Diagnostic> {
    let package_relative = package_relative_library_path(package_root, source_path)?;
    let parent = package_relative.parent().unwrap_or_else(|| Path::new(""));
    let stem = package_relative
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("source");
    Ok(package_root
        .join("test/generated")
        .join(parent)
        .join(format!("{stem}_test.dart")))
}

/// Returns a `package:` import URI for a source file under `lib/`.
pub fn package_import_uri(
    package_name: &str,
    package_root: &Path,
    source_path: &Path,
) -> Result<String, Diagnostic> {
    let package_relative = package_relative_library_path(package_root, source_path)?;
    Ok(format!(
        "package:{package_name}/{}",
        package_relative
            .to_string_lossy()
            .replace('\\', "/")
            .trim_start_matches('/')
    ))
}

/// Rewrites a library import into a package import when it is relative to the source file.
pub fn rewrite_library_import_uri(
    package_name: &str,
    package_root: &Path,
    source_path: &Path,
    import_uri: &str,
) -> Result<String, Diagnostic> {
    if import_uri.starts_with("package:") || import_uri.starts_with("dart:") {
        return Ok(import_uri.to_owned());
    }

    let source_dir = normalized_source_path(package_root, source_path)?
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("lib"));
    let resolved = normalize_relative_path(source_dir.join(import_uri));
    Ok(format!(
        "package:{package_name}/{}",
        resolved
            .strip_prefix("lib")
            .map_err(|_| {
                Diagnostic::error(format!(
                    "generated test import `{import_uri}` for `{}` must resolve under lib/",
                    source_path.display()
                ))
            })?
            .to_string_lossy()
            .replace('\\', "/")
            .trim_start_matches('/')
    ))
}

fn library_relative_path(package_root: &Path, source_path: &Path) -> Result<PathBuf, Diagnostic> {
    let normalized = normalized_source_path(package_root, source_path)?;
    if !normalized.starts_with("lib") {
        return Err(Diagnostic::error(format!(
            "Dust source `{}` must live under lib/",
            source_path.display()
        )));
    }
    Ok(normalized)
}

fn package_relative_library_path(
    package_root: &Path,
    source_path: &Path,
) -> Result<PathBuf, Diagnostic> {
    library_relative_path(package_root, source_path)
        .map(|path| path.strip_prefix("lib").unwrap_or(&path).to_path_buf())
}

fn normalized_source_path(package_root: &Path, source_path: &Path) -> Result<PathBuf, Diagnostic> {
    let relative = if source_path.is_absolute() {
        source_path
            .strip_prefix(package_root)
            .map(Path::to_path_buf)
            .map_err(|_| {
                Diagnostic::error(format!(
                    "source `{}` is not inside package root `{}`",
                    source_path.display(),
                    package_root.display()
                ))
            })?
    } else {
        source_path.to_path_buf()
    };
    Ok(normalize_relative_path(relative))
}

fn normalize_relative_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
