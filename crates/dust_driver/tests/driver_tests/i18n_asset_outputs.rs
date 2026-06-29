use dust_diagnostics::Severity;
use dust_driver::{I18nBuildRequest, I18nCheckRequest, run_i18n_build, run_i18n_check};

use super::support::{make_workspace, write_file};

#[test]
fn i18n_build_warns_when_pubspec_misses_generated_assets() {
    let workspace = make_i18n_workspace("");

    let result = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == Severity::Warning
            && diagnostic.message.contains("assets/i18n/en/shop.arb")
    }));
}

#[test]
fn i18n_build_warns_when_pubspec_assets_cannot_be_read() {
    let workspace = make_i18n_workspace("");
    write_file(&workspace.path().join("pubspec.yaml"), "name: [\n");

    let result = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == Severity::Warning
            && diagnostic.message.contains("failed to parse pubspec")
    }));
}

#[test]
fn i18n_check_errors_when_pubspec_misses_arb_assets() {
    let workspace = make_i18n_workspace("");
    write_complete_shop_arb(&workspace);

    let result = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(result.has_errors());
    assert!(diagnostic_messages(&result).contains("assets/i18n/en/shop.arb"));
}

#[test]
fn i18n_check_rejects_parent_i18n_asset_directory() {
    let workspace = make_i18n_workspace("    - assets/i18n/\n");
    write_complete_shop_arb(&workspace);

    let result = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(result.has_errors());
    assert!(diagnostic_messages(&result).contains("assets/i18n/en/shop.arb"));
}

#[test]
fn i18n_check_accepts_explicit_files_and_locale_directories() {
    let workspace = make_i18n_workspace(
        r"    - .\assets\i18n\en\
    - assets/i18n/my/
",
    );
    write_complete_shop_arb(&workspace);

    let result = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
}

fn make_i18n_workspace(assets: &str) -> tempfile::TempDir {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my]\n",
    );
    write_file(
        &workspace.path().join("pubspec.yaml"),
        &format!("name: dust_test\nflutter:\n  assets:\n{assets}"),
    );
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build() {
  const TranslatedText('shop_title', defaultText: 'Shop');
}
"#,
    );
    workspace
}

fn write_complete_shop_arb(workspace: &tempfile::TempDir) {
    write_file(
        &workspace.path().join("assets/i18n/en/shop.arb"),
        r#"{
  "@@locale": "en",
  "title": "Shop",
  "@title": {
    "description": "Translation for `shop_title`."
  }
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/my/shop.arb"),
        r#"{
  "@@locale": "my",
  "title": "Localized Shop",
  "@title": {
    "description": "Translation for `shop_title`."
  }
}
"#,
    );
}

fn diagnostic_messages(result: &dust_driver::CommandResult) -> String {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}
