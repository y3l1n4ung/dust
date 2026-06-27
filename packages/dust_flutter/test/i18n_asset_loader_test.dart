import 'dart:convert';

import 'package:dust_flutter/i18n.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('loads ARB assets and ignores metadata and non-string values', () async {
    final controller = I18nController(
      config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
    );
    final assets = _MemoryAssetBundle({
      'assets/i18n/en/home.arb': '''
{
  "@@locale": "en",
  "title": "Home",
  "@title": {
    "description": "Home page title"
  },
  "home.footer": "Footer",
  "count": 3
}
''',
    });

    final bundle = await controller.loadAssetBundle(
      locale: 'en',
      namespace: 'home',
      assetBundle: assets,
    );

    expect(bundle.messages, {'title': 'Home', 'home.footer': 'Footer'});
    expect(controller.translate('home.title'), 'Home');
    expect(controller.translate('home.footer'), 'Footer');
    expect(controller.translate('home.count'), 'home.count');
  });

  test('loads locale and fallback ARB assets with a custom pattern', () async {
    final controller = I18nController(
      config: const I18nConfig(locales: ['en', 'my'], fallbackLocale: 'en'),
      locale: 'my',
    );
    final assets = _MemoryAssetBundle({
      'l10n/en/home.arb': '{"title":"Home","subtitle":"Welcome"}',
      'l10n/my/home.arb': '{"title":"အိမ်"}',
    });

    final loaded = await controller.loadAssetBundles(
      namespaces: ['home'],
      assetBundle: assets,
      assetPattern: 'l10n/{locale}/{namespace}.arb',
    );

    expect(loaded.length, 2);
    expect(controller.translate('home.title'), 'အိမ်');
    expect(controller.translate('home.subtitle'), 'Welcome');
  });

  test('rejects non-object ARB assets', () {
    expect(
      () => I18nArbParser.parse(
        '[]',
        locale: 'en',
        namespace: 'home',
      ),
      throwsFormatException,
    );
  });
}

final class _MemoryAssetBundle extends CachingAssetBundle {
  _MemoryAssetBundle(this.assets);

  final Map<String, String> assets;

  @override
  Future<ByteData> load(String key) async {
    final source = assets[key];
    if (source == null) {
      throw StateError('Missing test asset: $key');
    }
    return ByteData.sublistView(Uint8List.fromList(utf8.encode(source)));
  }
}
