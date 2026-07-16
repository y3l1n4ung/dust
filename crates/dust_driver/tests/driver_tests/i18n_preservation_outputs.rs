use std::fs;

use dust_driver::{I18nBuildRequest, run_i18n_build};
use serde_json::{Value, json};

use super::support::{make_workspace, write_file};

#[test]
fn i18n_build_preserves_existing_chinese_values_and_metadata() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, zh]\n",
    );
    write_file(
        &workspace.path().join("lib/recording.dart"),
        r#"
import 'package:dust_flutter/i18n.dart';

void build(count) {
  const TranslatedText(
    'ppg_no_recording',
    defaultText: 'No recordings available',
  );
  TranslatedText(
    'ppg_recording_count',
    defaultText: '{count} recordings available',
    args: {'count': count},
  );
  const TranslatedText('ppg_start_recording', defaultText: 'Start recording');
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/en/ppg.arb"),
        r#"{
  "@@locale": "en",
  "no_recording": "No recording yet"
}
"#,
    );
    write_file(
        &workspace.path().join("assets/i18n/zh/ppg.arb"),
        r#"{
  "@@locale": "zh",
  "no_recording": "还没有录音",
  "recording_count": "共有 {count} 条录音",
  "@recording_count": {
    "description": "现有录音计数",
    "placeholders": {
      "count": {
        "type": "int",
        "example": "42"
      }
    }
  },
  "unrelated": "保留此翻译"
}
"#,
    );

    let result = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
        ..Default::default()
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let en = read_arb(&workspace.path().join("assets/i18n/en/ppg.arb"));
    let zh = read_arb(&workspace.path().join("assets/i18n/zh/ppg.arb"));
    assert_eq!(en["no_recording"], "No recording yet");
    assert_eq!(zh["no_recording"], "还没有录音");
    assert_eq!(zh["recording_count"], "共有 {count} 条录音");
    assert_eq!(zh["unrelated"], "保留此翻译");
    assert_eq!(zh["start_recording"], "");
    assert_eq!(
        zh["@recording_count"],
        json!({
            "description": "现有录音计数",
            "placeholders": {
                "count": {
                    "type": "int",
                    "example": "42"
                }
            }
        })
    );

    let preview = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
        sync_source: true,
        dry_run: true,
    });
    assert!(!preview.has_errors(), "{:?}", preview.diagnostics);
    assert_eq!(preview.i18n_build.as_ref().unwrap().synced_messages, 1);
    assert!(preview.i18n_build.as_ref().unwrap().dry_run);
    assert_eq!(
        read_arb(&workspace.path().join("assets/i18n/en/ppg.arb"))["no_recording"],
        "No recording yet"
    );

    let synced = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
        sync_source: true,
        dry_run: false,
    });
    assert!(!synced.has_errors(), "{:?}", synced.diagnostics);
    assert_eq!(synced.i18n_build.as_ref().unwrap().synced_messages, 1);
    let en = read_arb(&workspace.path().join("assets/i18n/en/ppg.arb"));
    let zh = read_arb(&workspace.path().join("assets/i18n/zh/ppg.arb"));
    assert_eq!(en["no_recording"], "No recordings available");
    assert_eq!(zh["no_recording"], "还没有录音");

    let no_op = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
        sync_source: true,
        dry_run: false,
    });
    assert!(!no_op.has_errors(), "{:?}", no_op.diagnostics);
    let report = no_op.i18n_build.as_ref().unwrap();
    assert_eq!(report.synced_messages, 0);
    assert_eq!(report.changed_files, 0);
}

fn read_arb(path: &std::path::Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}
