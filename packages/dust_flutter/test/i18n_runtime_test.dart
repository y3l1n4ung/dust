import 'package:dust_flutter/i18n.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('controller follows i18n lookup order', () {
    final controller = I18nController(
      config: const I18nConfig(
        locales: ['en', 'my'],
        fallbackLocale: 'en',
      ),
      locale: 'my',
      bundles: const [
        I18nBundle(
          locale: 'en',
          namespace: 'home',
          messages: {
            'title': 'Home',
            'subtitle': 'Welcome',
            'greeting': 'Hello {name}',
          },
        ),
        I18nBundle(
          locale: 'my',
          namespace: 'home',
          messages: {'title': 'အိမ်'},
        ),
      ],
    );

    expect(controller.translate('home_title'), 'အိမ်');
    expect(controller.translate('home_subtitle'), 'Welcome');
    expect(
      controller.translate(
        'home_missing',
        defaultText: 'Missing {name}',
        args: {'name': 'May'},
      ),
      'Missing May',
    );
    expect(controller.translate('home_none'), 'home_none');

    controller.setOverride('home_title', 'Override {name}');
    expect(
      controller.translate('home_title', args: {'name': 'Aye'}),
      'Override Aye',
    );

    controller
      ..clearOverrides()
      ..setLocale('en');
    expect(controller.translate('home_title'), 'Home');
  });

  test('controller rejects unsupported locales', () {
    final controller = I18nController(
      config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
    );

    expect(() => controller.setLocale('my'), throwsArgumentError);
  });
}
