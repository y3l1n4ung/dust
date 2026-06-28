import 'dart:convert';

import 'package:dust_flutter/i18n.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('loads ARB assets and ignores metadata and non-string values', () async {
    final controller = I18nController(
      config: const I18nConfig(
        locales: ['en'],
        fallbackLocale: 'en',
      ),
    );
    final assets = _MemoryAssetBundle({
      'assets/i18n/en/home.arb': '''
{
  "@@locale": "en",
  "title": "Home",
  "@title": {
    "description": "Home page title"
  },
  "footer": "Footer",
  "count": 3
}
''',
    });

    final bundle = await controller.loadAssetBundle(
      locale: 'en',
      namespace: 'home',
      assetBundle: assets,
    );

    expect(bundle.messages, {'title': 'Home', 'footer': 'Footer'});
    expect(controller.translate('home_title'), 'Home');
    expect(controller.translate('home_footer'), 'Footer');
    expect(controller.translate('home_count'), 'home_count');
  });

  test('loads locale and fallback ARB assets with a custom pattern', () async {
    final controller = I18nController(
      config: const I18nConfig(
        locales: ['en', 'my'],
        fallbackLocale: 'en',
      ),
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
    expect(controller.translate('home_title'), 'အိမ်');
    expect(controller.translate('home_subtitle'), 'Welcome');
  });

  test('discovers namespaces from the asset manifest when omitted', () async {
    final controller = I18nController(
      config: const I18nConfig(
        locales: ['en'],
        fallbackLocale: 'en',
      ),
    );
    final assets = _MemoryAssetBundle({
      'assets/i18n/en/login.arb': '{"title":"Sign in"}',
      'assets/i18n/en/shop.arb': '{"title":"Shop"}',
      'assets/images/logo.png': '',
    });

    final loaded = await controller.loadAssetBundles(assetBundle: assets);

    expect(loaded.length, 2);
    expect(controller.translate('login_title'), 'Sign in');
    expect(controller.translate('shop_title'), 'Shop');
  });

  test('fails when namespace discovery finds no ARB assets', () async {
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          ..setMockMessageHandler('flutter/assets', (message) async {
            if (message == null) return null;
            final key = utf8.decode(
              message.buffer.asUint8List(
                message.offsetInBytes,
                message.lengthInBytes,
              ),
            );
            if (key != 'AssetManifest.bin') return null;
            return _assetManifestFor(['assets/images/logo.png']);
          });
    addTearDown(() {
      messenger.setMockMessageHandler('flutter/assets', null);
    });
    final controller = I18nController(
      config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
    );

    await expectLater(controller.loadAssetBundles(), throwsStateError);
  });

  test('parses only string ARB messages from object assets', () {
    final bundle = I18nArbParser.parse(
      '{"title":"Home","enabled":true,"@title":{"description":"Title"}}',
      locale: 'en',
      namespace: 'home',
    );

    expect(bundle.messages, {'title': 'Home'});
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
    if (key == 'AssetManifest.bin') {
      return _assetManifest();
    }
    final source = assets[key];
    if (source == null) {
      throw StateError('Missing test asset: $key');
    }
    return ByteData.sublistView(Uint8List.fromList(utf8.encode(source)));
  }

  ByteData _assetManifest() {
    return _assetManifestFor(assets.keys);
  }
}

ByteData _assetManifestFor(Iterable<String> assets) {
  final manifest = {
    for (final key in assets)
      key: [
        {'asset': key},
      ],
  };
  final encoded = const StandardMessageCodec().encodeMessage(manifest);
  if (encoded == null) {
    throw StateError('Failed to encode test asset manifest.');
  }
  return encoded;
}
