use std::fs;

use dust_driver::{
    BuildRequest, I18nBuildRequest, I18nCheckRequest, run_build, run_i18n_build, run_i18n_check,
};

use super::support::{generated_output, make_workspace, write_file};

/// ARB file creation, translation preservation and key validation.
#[path = "i18n_outputs/arb.rs"]
mod arb;

#[test]
fn build_writes_generated_i18n_bootstrap() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my]\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(result.build_artifacts.len(), 1);
    assert_eq!(
        result.build_artifacts[0].output_path,
        workspace.path().join("lib/i18n/app_i18n.g.dart")
    );

    let output = fs::read_to_string(workspace.path().join("lib/i18n/app_i18n.g.dart")).unwrap();
    assert_eq!(
        output,
        generated_output(
            r#"import 'dart:async' show unawaited;

import 'package:dust_flutter/i18n.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter/widgets.dart';

const List<String> appI18nLocales = <String>['en', 'my'];
const List<Locale> appI18nSupportedLocales = <Locale>[
  Locale.fromSubtags(languageCode: 'en'),
  Locale.fromSubtags(languageCode: 'my'),
];
const List<LocalizationsDelegate<dynamic>> appI18nLocalizationsDelegates =
    <LocalizationsDelegate<dynamic>>[
  GlobalMaterialLocalizations.delegate,
  GlobalCupertinoLocalizations.delegate,
  GlobalWidgetsLocalizations.delegate,
];
const String appI18nFallbackLocale = 'en';
const String appI18nAssetPattern = defaultI18nAssetPattern;

Locale appI18nLocaleOf(String locale) {
  switch (locale) {
    case 'en':
      return Locale.fromSubtags(languageCode: 'en');
    case 'my':
      return Locale.fromSubtags(languageCode: 'my');
    default:
      return Locale(locale);
  }
}

const I18nConfig appI18nConfig = I18nConfig(
  locales: appI18nLocales,
  fallbackLocale: appI18nFallbackLocale,
);

class AppI18n extends StatefulWidget {
  const AppI18n({
    required this.child,
    this.assetBundle,
    super.key,
  });

  final Widget child;
  final AssetBundle? assetBundle;

  @override
  State<AppI18n> createState() => _AppI18nState();
}

class _AppI18nState extends State<AppI18n> {
  late final I18nController _controller =
      I18nController(config: appI18nConfig);

  @override
  void initState() {
    super.initState();
    unawaited(_loadBundles());
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return I18nScope(
      controller: _controller,
      child: widget.child,
    );
  }

  Future<void> _loadBundles() async {
    try {
      await _controller.loadAssetBundles(
        assetBundle: widget.assetBundle,
        assetPattern: appI18nAssetPattern,
      );
    } catch (error, stackTrace) {
      FlutterError.reportError(
        FlutterErrorDetails(
          exception: error,
          stack: stackTrace,
          library: 'dust_flutter',
          context: ErrorDescription('while loading i18n assets'),
        ),
      );
    }
  }
}
"#
        )
    );
}

#[test]
fn build_writes_i18n_bootstrap_locale_subtags() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en_US, en_US_POSIX, zh_Hans, zh_Hans_CN]\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    let output = fs::read_to_string(workspace.path().join("lib/i18n/app_i18n.g.dart")).unwrap();
    assert!(output.contains(
        "const List<String> appI18nLocales = <String>['en_US', 'en_US_POSIX', 'zh_Hans', 'zh_Hans_CN'];"
    ));
    assert!(output.contains("Locale.fromSubtags(languageCode: 'en', countryCode: 'US')"));
    assert!(!output.contains("scriptCode: 'US'"));
    assert!(!output.contains("countryCode: 'POSIX'"));
    assert!(output.contains("Locale.fromSubtags(languageCode: 'zh', scriptCode: 'Hans')"));
    assert!(
        output.contains(
            "Locale.fromSubtags(languageCode: 'zh', scriptCode: 'Hans', countryCode: 'CN')"
        )
    );
    assert!(output.contains("case 'zh_Hans_CN':"));
}

#[test]
fn i18n_build_synchronizes_opt_in_ios_plist_without_touching_custom_keys() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("dust.yaml"),
        "i18n:\n  locales: [en, my, zh_Hans_CN]\n  ios:\n    info_plist: ios/Runner/Info.plist\n    sync_development_region: true\n",
    );
    write_file(
        &workspace.path().join("ios/Runner/Info.plist"),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\">\n<dict>\n\t<key>CFBundleDevelopmentRegion</key>\n\t<string>$(DEVELOPMENT_LANGUAGE)</string>\n\t<key>CFBundleDisplayName</key>\n\t<string>Fixture</string>\n</dict>\n</plist>\n",
    );
    write_file(
        &workspace.path().join("lib/home.dart"),
        "import 'package:dust_flutter/i18n.dart';\nvoid build() { const TranslatedText('shop_title', defaultText: 'Shop'); }\n",
    );

    let result = run_i18n_build(I18nBuildRequest {
        cwd: workspace.path().to_path_buf(),
        ..Default::default()
    });
    assert!(!result.has_errors(), "{:?}", result.diagnostics);

    let plist = fs::read_to_string(workspace.path().join("ios/Runner/Info.plist")).unwrap();
    assert!(plist.contains("<string>en</string>"));
    assert!(plist.contains("<string>my</string>"));
    assert!(plist.contains("<string>zh-Hans-CN</string>"));
    assert!(plist.contains("<key>CFBundleDisplayName</key>"));
    assert!(!plist.contains("$(DEVELOPMENT_LANGUAGE)"));
    assert!(plist.contains("<string>Fixture</string>"));

    for (locale, translation) in [("my", "ဆိုင်"), ("zh_Hans_CN", "商店")] {
        let path = workspace
            .path()
            .join(format!("assets/i18n/{locale}/shop.arb"));
        let translated = fs::read_to_string(&path)
            .unwrap()
            .replace("\"title\": \"\"", &format!("\"title\": \"{translation}\""));
        fs::write(path, translated).unwrap();
    }
    write_file(
        &workspace.path().join("pubspec.yaml"),
        "name: dust_test\nflutter:\n  assets:\n    - assets/i18n/en/\n    - assets/i18n/my/\n    - assets/i18n/zh_Hans_CN/\n",
    );

    let check = run_i18n_check(I18nCheckRequest {
        cwd: workspace.path().to_path_buf(),
    });
    assert!(!check.has_errors(), "{:?}", check.diagnostics);
}
