use std::fs;

use dust_driver::{BuildRequest, run_build};

use super::support::{generated_output, make_workspace, write_file};

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
import 'package:flutter/widgets.dart';

const List<String> appI18nLocales = <String>['en', 'my'];
const String appI18nFallbackLocale = 'en';
const String appI18nAssetPattern = defaultI18nAssetPattern;

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
