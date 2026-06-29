use dust_cli::run_cli;

use super::helpers::{make_workspace, write_file};

#[test]
fn cli_i18n_build_writes_arb_files() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my]\n",
    );
    write_file(
        &workspace.path().join("pubspec.yaml"),
        "name: dust_test\nflutter:\n  assets:\n    - assets/i18n/en/\n    - assets/i18n/my/\n",
    );
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build(count) {
  TranslatedText(
    'shop_item_count',
    defaultText: '{count} items',
    args: {'count': count},
  );
}
"#,
    );

    let run = run_cli([
        "i18n",
        "build",
        "--root",
        workspace.path().to_str().unwrap(),
    ]);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert!(run.stderr.is_empty());
    assert!(
        run.stdout
            .contains("i18n build  files: 2  changed: 2  keys: 1  added: 2")
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("assets/i18n/en/shop.arb")).unwrap(),
        "{\n  \"@@locale\": \"en\",\n  \"@@context\": \"Translations for `shop` namespace.\",\n  \"item_count\": \"{count} items\",\n  \"@item_count\": {\n    \"description\": \"Translation for `shop_item_count`.\",\n    \"placeholders\": {\n      \"count\": {\n        \"example\": \"1\"\n      }\n    }\n  }\n}\n"
    );
}

#[test]
fn cli_i18n_check_reports_clean_arb_files() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my]\n",
    );
    write_file(
        &workspace.path().join("pubspec.yaml"),
        "name: dust_test\nflutter:\n  assets:\n    - assets/i18n/en/\n    - assets/i18n/my/\n",
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

    let run = run_cli([
        "i18n",
        "check",
        "--root",
        workspace.path().to_str().unwrap(),
    ]);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert!(run.stderr.is_empty());
    assert!(
        run.stdout
            .contains("i18n check  files: 2  keys: 1  checked: 2  stale: 0")
    );
}

#[test]
fn cli_i18n_scan_reports_static_keys() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build(context, cart, user) {
  TranslatedText(
    'home_cart',
    defaultText: 'Cart {count}',
    args: {
      'count': cart.count,
      'name': user.name,
      'count': cart.total,
    },
  );
  context.tr('home_cart', args: {'total': cart.total});
  context.tr('home_title', defaultText: 'Home');
  TranslatedText.dynamic(product.titleKey, fallback: product.title);
}
"#,
    );

    let run = run_cli(["i18n", "scan", "--root", workspace.path().to_str().unwrap()]);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert!(run.stderr.is_empty());
    assert_eq!(
        strip_banner_and_time(&run.stdout),
        "i18n scan  files: 1  keys: 2  time: <ms>\n\
home_cart  namespace=home  default=\"Cart {count}\"  args=count,name,total\n\
home_title  namespace=home  default=\"Home\"  args=-\n"
    );
}

#[test]
fn cli_i18n_scan_warns_on_runtime_static_keys() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build(context, dynamicKey) {
  TranslatedText(dynamicKey);
  context.tr(dynamicKey);
}
"#,
    );

    let run = run_cli(["i18n", "scan", "--root", workspace.path().to_str().unwrap()]);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert!(run.stderr.is_empty());
    assert!(run.stdout.contains("i18n scan  files: 1  keys: 0"));
    assert!(
        run.stdout
            .contains("diagnostics  errors: 0  warnings: 2  notes: 0")
    );
    assert!(
        run.stdout
            .contains("TranslatedText requires a string literal key")
    );
    assert!(
        run.stdout
            .contains("context.tr requires a string literal key")
    );
}

#[test]
fn cli_i18n_scan_warns_on_hardcoded_text_literals() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/home.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';
import 'package:flutter/widgets.dart';

void build(context) {
  const Text('Checkout');
  Text(context.tr('shop_title'));
}
"#,
    );

    let run = run_cli(["i18n", "scan", "--root", workspace.path().to_str().unwrap()]);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert!(run.stderr.is_empty());
    assert!(run.stdout.contains("i18n scan  files: 1  keys: 1"));
    assert!(
        run.stdout
            .contains("diagnostics  errors: 0  warnings: 1  notes: 0")
    );
    assert!(
        run.stdout
            .contains("hardcoded Text string; use TranslatedText or context.tr")
    );
}

fn strip_banner_and_time(source: &str) -> String {
    let mut lines = source
        .split_once("\n\n")
        .map_or(source, |(_, body)| body)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if let Some(first) = lines.first_mut() {
        if let Some((prefix, _)) = first.rsplit_once("time: ") {
            *first = format!("{prefix}time: <ms>");
        }
    }
    format!("{}\n", lines.join("\n"))
}
