use std::{collections::BTreeSet, path::Path};

use dust_diagnostics::Diagnostic;
use dust_workspace::{I18nConfig, load_flutter_assets};

use crate::i18n_keys::I18nPlannedEntry;

/// Severity used for pubspec asset declaration diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum I18nAssetSeverity {
    /// Missing assets should block the command.
    Error,
    /// Missing assets should be reported without blocking the command.
    Warning,
}

/// Validates that all planned ARB files are declared as Flutter assets.
pub(crate) fn validate_i18n_asset_declarations(
    package_root: &Path,
    config: &I18nConfig,
    entries: &[I18nPlannedEntry],
    severity: I18nAssetSeverity,
) -> Result<Vec<Diagnostic>, Diagnostic> {
    let declarations = load_flutter_assets(package_root)?
        .into_iter()
        .map(|asset| normalize_asset_path(&asset))
        .collect::<Vec<_>>();
    let namespaces = entries
        .iter()
        .map(|entry| entry.namespace.as_str())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();

    for locale in &config.locales {
        for namespace in &namespaces {
            let asset = format!("assets/i18n/{locale}/{namespace}.arb");
            if is_asset_declared(&declarations, &asset) {
                continue;
            }
            diagnostics.push(missing_asset_diagnostic(&asset, severity));
        }
    }

    Ok(diagnostics)
}

/// Returns whether an expected file is covered by Flutter asset declarations.
fn is_asset_declared(declarations: &[String], expected: &str) -> bool {
    declarations.iter().any(|declaration| {
        declaration == expected
            || declaration.ends_with('/')
                && expected
                    .strip_prefix(declaration)
                    .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
    })
}

/// Normalizes a pubspec asset path for comparison.
fn normalize_asset_path(asset: &str) -> String {
    let normalized = asset.trim().replace('\\', "/");
    normalized.trim_start_matches("./").to_owned()
}

/// Builds one missing-asset diagnostic.
fn missing_asset_diagnostic(asset: &str, severity: I18nAssetSeverity) -> Diagnostic {
    let message =
        format!("i18n ARB asset `{asset}` must be declared in pubspec.yaml flutter.assets");
    let note = format!(
        "Declare `{asset}` or its locale directory `{}/`.",
        locale_directory(asset)
    );
    match severity {
        I18nAssetSeverity::Error => Diagnostic::error(message).with_note(note),
        I18nAssetSeverity::Warning => Diagnostic::warning(message).with_note(note),
    }
}

/// Returns the direct locale asset directory for an ARB asset.
fn locale_directory(asset: &str) -> &str {
    asset
        .rsplit_once('/')
        .map(|(directory, _)| directory)
        .unwrap_or(asset)
}

#[cfg(test)]
mod tests {
    use super::is_asset_declared;

    #[test]
    fn asset_declaration_matches_files_and_direct_directories() {
        let declarations = vec![
            "assets/i18n/en/shop.arb".to_owned(),
            "assets/i18n/my/".to_owned(),
        ];

        assert!(is_asset_declared(&declarations, "assets/i18n/en/shop.arb"));
        assert!(is_asset_declared(&declarations, "assets/i18n/my/shop.arb"));
    }

    #[test]
    fn asset_declaration_rejects_parent_directories() {
        let declarations = vec!["assets/i18n/".to_owned()];

        assert!(!is_asset_declared(&declarations, "assets/i18n/en/shop.arb"));
    }
}
