use dust_diagnostics::Severity;
use dust_text::{FileId, SourceText};

use super::{I18nTranslationKind, scan_i18n_source};

#[test]
fn scans_static_translated_text_and_context_keys() {
    let source = source_text(
        r#"
Widget build(BuildContext context) {
  return Column(children: [
    const TranslatedText('home_title'),
    TranslatedText(
      'home_greeting',
      defaultText: 'Hello {name}',
      args: const {'name': user.name},
    ),
    Text(context.tr("settings_title", defaultText: "Settings")),
  ]);
}
"#,
    );

    let result = scan_i18n_source(&source);

    assert!(result.diagnostics.is_empty());
    assert_eq!(result.entries.len(), 3);
    assert_eq!(result.entries[0].key, "home_title");
    assert_eq!(result.entries[0].namespace, "home");
    assert_eq!(result.entries[0].kind, I18nTranslationKind::TranslatedText);
    assert_eq!(result.entries[1].key, "home_greeting");
    assert_eq!(
        result.entries[1].default_text.as_deref(),
        Some("Hello {name}")
    );
    assert_eq!(result.entries[1].args, vec!["name"]);
    assert_eq!(result.entries[2].key, "settings_title");
    assert_eq!(result.entries[2].namespace, "settings");
    assert_eq!(result.entries[2].kind, I18nTranslationKind::ContextTr);
}

#[test]
fn ignores_dynamic_api_and_warns_on_runtime_static_keys() {
    let source = source_text(
        r#"
Widget build(BuildContext context) {
  return Column(children: [
    TranslatedText.dynamic(product.titleKey, fallback: product.title),
    TranslatedText(product.titleKey),
    Text(context.tr(settingsKey)),
  ]);
}
"#,
    );

    let result = scan_i18n_source(&source);

    assert!(result.entries.is_empty());
    assert_eq!(result.diagnostics.len(), 2);
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    assert_eq!(
        result.diagnostics[0].message,
        "TranslatedText requires a string literal key; use TranslatedText.dynamic for runtime keys"
    );
    assert_eq!(
        result.diagnostics[1].message,
        "context.tr requires a string literal key"
    );
}

#[test]
fn keeps_placeholder_keys_deterministic_and_unique() {
    let source = source_text(
        r#"
Widget build(BuildContext context) {
  return TranslatedText(
    'home_cart',
    args: {
      'count': cart.count,
      "name": user.name,
      'count': other,
    },
  );
}
"#,
    );

    let result = scan_i18n_source(&source);

    assert!(result.diagnostics.is_empty());
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].args, vec!["count", "name"]);
}

#[test]
fn warns_on_direct_hardcoded_text_literals() {
    let source = source_text(
        r#"
Widget build(BuildContext context) {
  return Column(children: [
    const Text('Checkout'),
    Text("Cart"),
    Text(context.tr('shop_title')),
  ]);
}
"#,
    );

    let result = scan_i18n_source(&source);

    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].key, "shop_title");
    assert_eq!(result.diagnostics.len(), 2);
    assert!(
        result
            .diagnostics
            .iter()
            .all(|item| item.severity == Severity::Warning)
    );
    assert!(
        result.diagnostics.iter().all(|item| {
            item.message == "hardcoded Text string; use TranslatedText or context.tr"
        })
    );
}

#[test]
fn scans_shopping_app_i18n_sample_without_dynamic_warnings() {
    let source = source_text(include_str!(
        "../../../../examples/shopping_app/lib/features/products/views/products_screen.dart"
    ));

    let result = scan_i18n_source(&source);
    let keys = result
        .entries
        .iter()
        .map(|entry| entry.key.as_str())
        .collect::<Vec<_>>();

    assert!(result.diagnostics.is_empty());
    assert!(keys.contains(&"shop_title"));
    assert!(keys.contains(&"shop_search_hint"));
    assert!(keys.contains(&"shop_no_products"));
    assert!(keys.contains(&"shop_product_price"));
    assert!(keys.contains(&"shop_rating_summary"));
    assert!(!keys.iter().any(|key| key.starts_with("shop_category_")));
    assert!(!keys.contains(&"shop_product_1_title"));
    assert!(!keys.contains(&"shop_product_2_title"));
}

fn source_text(text: &str) -> SourceText {
    SourceText::new(FileId::new(0), text)
}
