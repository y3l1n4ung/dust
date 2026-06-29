use std::fs;

use dust_driver::{I18nBuildRequest, run_i18n_build};
use serde_json::Value;

use super::support::{make_workspace, write_file};

const KEY_COUNT: usize = 10_000;

#[test]
fn i18n_build_handles_ten_thousand_static_keys() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my]\n",
    );
    write_file(&workspace.path().join("lib/home.dart"), &source_with_keys());

    let result = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let report = result.i18n_build.unwrap();
    assert_eq!(report.scanned_files, 1);
    assert_eq!(report.keys, KEY_COUNT);
    assert_eq!(report.arb_files, 2);
    assert_eq!(report.changed_files, 2);
    assert_eq!(report.added_messages, KEY_COUNT * 2);

    let en = read_arb(&workspace.path().join("assets/i18n/en/bench.arb"));
    let my = read_arb(&workspace.path().join("assets/i18n/my/bench.arb"));
    assert_eq!(en["@@locale"], "en");
    assert_eq!(en["@@context"], "Translations for `bench` namespace.");
    assert_eq!(en["message_00000"], "Message 0");
    assert_eq!(
        en["@message_00000"]["description"],
        "Translation for `bench_message_00000`."
    );
    assert_eq!(en["message_09999"], "Message 9999");
    assert_eq!(my["@@locale"], "my");
    assert_eq!(my["message_00000"], "");
    assert_eq!(my["message_09999"], "");
}

fn source_with_keys() -> String {
    let mut source = String::from("import 'package:dust_flutter/i18n.dart';\n\nvoid build() {\n");
    for index in 0..KEY_COUNT {
        source.push_str(&format!(
            "  const TranslatedText('bench_message_{index:05}', defaultText: 'Message {index}');\n"
        ));
    }
    source.push_str("}\n");
    source
}

fn read_arb(path: &std::path::Path) -> Value {
    let source = fs::read_to_string(path).unwrap();
    serde_json::from_str(&source).unwrap()
}
