import 'package:dust_flutter/i18n.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('applies remote override bundle and rebuilds text', (
    tester,
  ) async {
    final controller = I18nController(
      config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
      bundles: const [
        I18nBundle(
          locale: 'en',
          namespace: 'shop',
          messages: {'title': 'Shop'},
        ),
      ],
    );

    await tester.pumpWidget(
      I18nScope(
        controller: controller,
        child: const Directionality(
          textDirection: TextDirection.ltr,
          child: TranslatedText('shop_title'),
        ),
      ),
    );

    expect(find.text('Shop'), findsOneWidget);

    controller.applyOverrideBundle(
      '{"title":"Remote Shop","@title":{"description":"ignored"},"count":2}',
      locale: 'en',
      namespace: 'shop',
    );
    await tester.pump();

    expect(find.text('Remote Shop'), findsOneWidget);
    expect(controller.translate('shop_count'), 'shop_count');
  });

  test('replaces stale overrides from the same namespace', () {
    final controller = I18nController(
      config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
      bundles: const [
        I18nBundle(
          locale: 'en',
          namespace: 'shop',
          messages: {'title': 'Shop', 'subtitle': 'Deals'},
        ),
      ],
    );

    final remoteController = controller
      ..applyOverrideBundle(
        '{"title":"Remote Shop","subtitle":"Remote Deals"}',
        locale: 'en',
        namespace: 'shop',
      );
    expect(remoteController.translate('shop_subtitle'), 'Remote Deals');

    final freshController = controller
      ..applyOverrideBundle(
        '{"title":"Fresh Shop"}',
        locale: 'en',
        namespace: 'shop',
      );

    expect(freshController.translate('shop_title'), 'Fresh Shop');
    expect(freshController.translate('shop_subtitle'), 'Deals');
  });
}
