//! ARB file creation, translation preservation and key validation.

use super::*;

#[test]
fn i18n_build_creates_arb_files_from_scanned_keys() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my]\n",
    );
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build(count) {
  const TranslatedText('shop_title', defaultText: 'Shop');
  TranslatedText(
    'shop_item_count',
    defaultText: '{count} items',
    args: {'count': count},
  );
}
"#,
    );

    let result = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
        ..Default::default()
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let report = result.i18n_build.unwrap();
    assert_eq!(report.scanned_files, 1);
    assert_eq!(report.keys, 2);
    assert_eq!(report.arb_files, 2);
    assert_eq!(report.changed_files, 2);
    assert_eq!(report.added_messages, 4);
    assert_eq!(result.build_artifacts.len(), 1);
    assert_eq!(
        fs::read_to_string(workspace.path().join("assets/i18n/en/shop.arb")).unwrap(),
        r#"{
  "@@locale": "en",
  "@@context": "Translations for `shop` namespace.",
  "item_count": "{count} items",
  "@item_count": {
    "description": "Translation for `shop_item_count`.",
    "placeholders": {
      "count": {
        "example": "1"
      }
    }
  },
  "title": "Shop",
  "@title": {
    "description": "Translation for `shop_title`."
  }
}
"#
    );
    assert_eq!(
        fs::read_to_string(workspace.path().join("assets/i18n/my/shop.arb")).unwrap(),
        r#"{
  "@@locale": "my",
  "@@context": "Translations for `shop` namespace.",
  "item_count": "",
  "@item_count": {
    "description": "Translation for `shop_item_count`.",
    "placeholders": {
      "count": {
        "example": "1"
      }
    }
  },
  "title": "",
  "@title": {
    "description": "Translation for `shop_title`."
  }
}
"#
    );
}

#[test]
fn i18n_build_preserves_existing_translations_and_metadata() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my]\n",
    );
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build() {
  const TranslatedText('shop_title', defaultText: 'Shop');
  const TranslatedText('shop_subtitle', defaultText: 'Daily deals');
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/en/shop.arb"),
        r#"{
  "@@locale": "en",
  "title": "Store",
  "@title": {
    "description": "Existing title metadata"
  }
}
"#,
    );

    let result = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
        ..Default::default()
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let report = result.i18n_build.unwrap();
    assert_eq!(report.changed_files, 2);
    assert_eq!(report.added_messages, 3);
    assert_eq!(
        fs::read_to_string(workspace.path().join("assets/i18n/en/shop.arb")).unwrap(),
        r#"{
  "@@locale": "en",
  "@@context": "Translations for `shop` namespace.",
  "subtitle": "Daily deals",
  "@subtitle": {
    "description": "Translation for `shop_subtitle`."
  },
  "title": "Store",
  "@title": {
    "description": "Existing title metadata"
  }
}
"#
    );
}

#[test]
fn i18n_build_rejects_non_arb_safe_key_shapes() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my]\n",
    );
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build() {
  const TranslatedText('shop.title', defaultText: 'Shop');
}
"#,
    );

    let result = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
        ..Default::default()
    });

    assert!(result.has_errors());
    assert!(
        result.diagnostics[0]
            .message
            .contains("use an underscore namespace prefix")
    );
    assert!(!workspace.path().join("assets/i18n/en/shop.arb").exists());
}
