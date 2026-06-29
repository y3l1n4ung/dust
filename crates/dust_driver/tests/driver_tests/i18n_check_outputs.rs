use std::fs;

use dust_driver::{I18nBuildRequest, I18nCheckRequest, run_i18n_build, run_i18n_check};

use super::support::{make_workspace, write_file};

#[test]
fn i18n_check_accepts_complete_arb_files() {
    let workspace = workspace_with_i18n_source();
    write_complete_shop_arb(&workspace);

    let result = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let report = result.i18n_check.unwrap();
    assert_eq!(report.scanned_files, 1);
    assert_eq!(report.keys, 2);
    assert_eq!(report.arb_files, 2);
    assert_eq!(report.checked_messages, 4);
    assert_eq!(report.stale_messages, 0);
}

#[test]
fn i18n_build_backfills_metadata_for_existing_messages() {
    let workspace = make_workspace();
    write_i18n_config(&workspace);
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build() {
  const TranslatedText('shop_title', defaultText: 'Shop');
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/en/shop.arb"),
        r#"{
  "@@locale": "en",
  "title": "Store"
}
"#,
    );

    let result = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(
        fs::read_to_string(workspace.path().join("assets/i18n/en/shop.arb")).unwrap(),
        r#"{
  "@@locale": "en",
  "@@context": "Translations for `shop` namespace.",
  "title": "Store",
  "@title": {
    "description": "Translation for `shop_title`."
  }
}
"#
    );
}

#[test]
fn i18n_check_reports_missing_file_key_and_metadata() {
    let workspace = make_workspace();
    write_i18n_config(&workspace);
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build() {
  const TranslatedText('shop_title', defaultText: 'Shop');
  const TranslatedText('shop_subtitle', defaultText: 'Deals');
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/en/shop.arb"),
        r#"{
  "@@locale": "en",
  "title": "Shop"
}
"#,
    );

    let result = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(result.has_errors());
    let messages = diagnostic_messages(&result);
    assert!(messages.contains("missing i18n ARB file"));
    assert!(messages.contains("missing i18n key `subtitle`"));
    assert!(messages.contains("must have `@title` metadata"));
}

#[test]
fn i18n_check_reports_invalid_arb_json_and_root_shape() {
    let workspace = workspace_with_i18n_source();
    write_file(&workspace.path().join("assets/i18n/en/shop.arb"), "{\n");
    write_file(&workspace.path().join("assets/i18n/my/shop.arb"), "[]\n");

    let result = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(result.has_errors());
    let messages = diagnostic_messages(&result);
    assert!(messages.contains("failed to parse i18n ARB file"));
    assert!(messages.contains("must contain a JSON object"));
}

#[test]
fn i18n_check_reports_empty_same_as_fallback_and_stale_keys() {
    let workspace = make_workspace();
    write_i18n_config(&workspace);
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build() {
  const TranslatedText('shop_title', defaultText: 'Shop');
  const TranslatedText('shop_subtitle', defaultText: 'Deals');
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/en/shop.arb"),
        r#"{
  "@@locale": "en",
  "old": "Old",
  "@old": {},
  "subtitle": "Deals",
  "@subtitle": {},
  "title": "Shop",
  "@title": {}
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/my/shop.arb"),
        r#"{
  "@@locale": "my",
  "subtitle": "",
  "@subtitle": {},
  "title": "Shop",
  "@title": {}
}
"#,
    );

    let result = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(result.has_errors());
    let messages = diagnostic_messages(&result);
    assert!(messages.contains("must not be empty"));
    assert!(messages.contains("matches fallback locale `en`"));
    assert!(messages.contains("stale i18n key `shop_old`"));
    assert_eq!(result.i18n_check.unwrap().stale_messages, 1);
}

#[test]
fn i18n_check_warns_on_english_looking_non_fallback_text() {
    let workspace = make_workspace();
    write_i18n_config(&workspace);
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build(price) {
  const TranslatedText('shop_title', defaultText: 'Shop');
  const TranslatedText('shop_subtitle', defaultText: 'Deals');
  TranslatedText('shop_price', defaultText: 'US$ {price}', args: {'price': price});
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/en/shop.arb"),
        r#"{
  "@@locale": "en",
  "price": "USD {price}",
  "@price": {
    "description": "Price",
    "placeholders": {
      "price": {
        "example": "9.99"
      }
    }
  },
  "subtitle": "Deals",
  "@subtitle": {
    "description": "Subtitle"
  },
  "title": "Shop",
  "@title": {
    "description": "Title"
  }
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/my/shop.arb"),
        r#"{
  "@@locale": "my",
  "price": "US$ {price}",
  "@price": {
    "description": "Price",
    "placeholders": {
      "price": {
        "example": "9.99"
      }
    }
  },
  "subtitle": "လျှော့စျေးများ",
  "@subtitle": {
    "description": "Subtitle"
  },
  "title": "Checkout now",
  "@title": {
    "description": "Title"
  }
}
"#,
    );

    let result = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let messages = diagnostic_messages(&result);
    assert!(messages.contains("looks like untranslated English text"));
    assert!(!messages.contains("i18n key `subtitle`"));
    assert!(!messages.contains("i18n key `price`"));
}

#[test]
fn i18n_check_reports_placeholder_metadata_and_message_mismatch() {
    let workspace = workspace_with_i18n_source();
    write_file(
        &workspace.path().join("assets/i18n/en/shop.arb"),
        r#"{
  "@@locale": "en",
  "item_count": "{count} items",
  "@item_count": {
    "placeholders": {
      "total": {}
    }
  },
  "title": "Shop",
  "@title": {}
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/my/shop.arb"),
        r#"{
  "@@locale": "my",
  "item_count": "{total} localized items",
  "@item_count": {
    "placeholders": {
      "count": {}
    }
  },
  "title": "Localized Shop",
  "@title": {}
}
"#,
    );

    let result = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(result.has_errors());
    let messages = diagnostic_messages(&result);
    assert!(messages.contains("has placeholder metadata"));
    assert!(messages.contains("uses placeholders"));
    assert!(messages.contains("must include a non-empty description"));
    assert!(messages.contains("must include a non-empty example"));
}

fn workspace_with_i18n_source() -> tempfile::TempDir {
    let workspace = make_workspace();
    write_i18n_config(&workspace);
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
    workspace
}

fn write_i18n_config(workspace: &tempfile::TempDir) {
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my]\n",
    );
}

fn write_complete_shop_arb(workspace: &tempfile::TempDir) {
    write_file(
        &workspace.path().join("assets/i18n/en/shop.arb"),
        r#"{
  "@@locale": "en",
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
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/my/shop.arb"),
        r#"{
  "@@locale": "my",
  "item_count": "{count} localized items",
  "@item_count": {
    "description": "Translation for `shop_item_count`.",
    "placeholders": {
      "count": {
        "example": "1"
      }
    }
  },
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
