use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use dust_diagnostics::Diagnostic;
use serde_json::Value;

use super::{is_dust_package, version::Version};

/// Resolved package roots keyed by Dust package name.
pub(super) fn resolved_dust_package_roots(
    package_config_path: &Path,
) -> Result<BTreeMap<String, PathBuf>, Diagnostic> {
    let contents = fs::read_to_string(package_config_path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read package configuration `{}`: {error}",
            package_config_path.display()
        ))
    })?;
    let value = serde_json::from_str::<Value>(&contents).map_err(|error| {
        Diagnostic::error(format!(
            "failed to parse package configuration `{}`: {error}",
            package_config_path.display()
        ))
    })?;
    let Some(packages) = value.get("packages").and_then(Value::as_array) else {
        return Ok(BTreeMap::new());
    };

    let mut roots = BTreeMap::new();
    for package in packages {
        let Some(name) = package.get("name").and_then(Value::as_str) else {
            continue;
        };
        if !is_dust_package(name) {
            continue;
        }
        let root_uri = package
            .get("rootUri")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                Diagnostic::error(format!(
                    "package configuration entry for `{name}` is missing `rootUri`"
                ))
            })?;
        roots.insert(
            name.to_owned(),
            resolve_package_root_uri(package_config_path, root_uri)?,
        );
    }

    Ok(roots)
}

/// Resolves a `package_config.json` root URI to a filesystem path.
fn resolve_package_root_uri(
    package_config_path: &Path,
    root_uri: &str,
) -> Result<PathBuf, Diagnostic> {
    if let Some(file_uri) = root_uri.strip_prefix("file://") {
        let local_path = file_uri
            .strip_prefix("localhost/")
            .map_or_else(|| file_uri.to_owned(), |path| format!("/{path}"));
        return Ok(PathBuf::from(percent_decode(&local_path)?));
    }

    let base = package_config_path
        .parent()
        .unwrap_or_else(|| Path::new(""));
    Ok(base.join(percent_decode(root_uri)?))
}

/// Percent-decodes a package configuration URI path.
fn percent_decode(value: &str) -> Result<String, Diagnostic> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hi = bytes.get(index + 1).copied();
            let lo = bytes.get(index + 2).copied();
            let Some(byte) = hi.zip(lo).and_then(|(hi, lo)| decode_hex_byte(hi, lo)) else {
                return Err(Diagnostic::error(format!(
                    "invalid percent escape in package root URI `{value}`"
                )));
            };
            decoded.push(byte);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded)
        .map_err(|error| Diagnostic::error(format!("package root URI is not valid UTF-8: {error}")))
}

/// Decodes one `%XX` byte from ASCII hex digits.
fn decode_hex_byte(hi: u8, lo: u8) -> Option<u8> {
    Some(hex_value(hi)? << 4 | hex_value(lo)?)
}

/// Converts one ASCII hex digit to its integer value.
fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Reads and parses one resolved Dust package pubspec version.
pub(super) fn read_package_version(
    package_root: &Path,
    package: &str,
) -> Result<(String, Version), Diagnostic> {
    let pubspec_path = package_root.join("pubspec.yaml");
    let contents = fs::read_to_string(&pubspec_path).map_err(|error| {
        Diagnostic::error(format!(
            "failed to read resolved `{package}` pubspec `{}`: {error}",
            pubspec_path.display()
        ))
    })?;
    let version_text = contents
        .lines()
        .find_map(pubspec_version_value)
        .ok_or_else(|| {
            Diagnostic::error(format!(
                "resolved `{package}` pubspec `{}` does not declare `version`",
                pubspec_path.display()
            ))
        })?;
    let version = Version::parse(&version_text).ok_or_else(|| {
        Diagnostic::error(format!(
            "resolved `{package}` has unsupported version `{version_text}`"
        ))
    })?;

    Ok((version_text, version))
}

/// Extracts a simple top-level `version:` value from a pubspec line.
fn pubspec_version_value(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let value = trimmed.strip_prefix("version:")?.split('#').next()?.trim();
    Some(value.trim_matches('"').trim_matches('\'').trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::resolve_package_root_uri;

    #[test]
    fn file_uri_resolution_keeps_localhost_paths_absolute() {
        let path = resolve_package_root_uri(
            Path::new("/workspace/.dart_tool/package_config.json"),
            "file://localhost/tmp/dust%20pkg",
        )
        .unwrap();

        assert_eq!(path, PathBuf::from("/tmp/dust pkg"));
    }
}
