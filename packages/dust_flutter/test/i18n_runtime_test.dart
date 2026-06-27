import 'package:dust_flutter/i18n.dart';
import 'package:flutter/widgets.dart';
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

    expect(controller.translate('home.title'), 'အိမ်');
    expect(controller.translate('home.subtitle'), 'Welcome');
    expect(
      controller.translate('home.missing',
          defaultText: 'Missing {name}', args: {'name': 'May'}),
      'Missing May',
    );
    expect(controller.translate('home.none'), 'home.none');

    controller.setOverride('home.title', 'Override {name}');
    expect(
      controller.translate('home.title', args: {'name': 'Aye'}),
      'Override Aye',
    );

    controller.clearOverrides();
    controller.setLocale('en');
    expect(controller.translate('home.title'), 'Home');
  });

  test('controller rejects unsupported locales', () {
    final controller = I18nController(
      config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
    );

    expect(() => controller.setLocale('my'), throwsArgumentError);
  });

  testWidgets('context.tr and TranslatedText rebuild when locale changes', (
    tester,
  ) async {
    final controller = I18nController(
      config: const I18nConfig(
        locales: ['en', 'my'],
        fallbackLocale: 'en',
      ),
      bundles: const [
        I18nBundle(
          locale: 'en',
          namespace: 'home',
          messages: {'title': 'Home', 'cta': 'Open {name}'},
        ),
        I18nBundle(
          locale: 'my',
          namespace: 'home',
          messages: {'title': 'အိမ်', 'cta': 'ဖွင့် {name}'},
        ),
      ],
    );

    await tester.pumpWidget(
      I18nScope(
        controller: controller,
        child: Directionality(
          textDirection: TextDirection.ltr,
          child: Column(
            children: [
              const TranslatedText('home.title'),
              Builder(
                builder: (context) {
                  return Text(
                    context.tr('home.cta', args: {'name': 'Dust'}),
                  );
                },
              ),
            ],
          ),
        ),
      ),
    );

    expect(find.text('Home'), findsOneWidget);
    expect(find.text('Open Dust'), findsOneWidget);

    controller.setLocale('my');
    await tester.pump();

    expect(find.text('အိမ်'), findsOneWidget);
    expect(find.text('ဖွင့် Dust'), findsOneWidget);
  });

  testWidgets('TranslatedText.dynamic uses fallback and overrides', (
    tester,
  ) async {
    final controller = I18nController(
      config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
    );

    await tester.pumpWidget(
      I18nScope(
        controller: controller,
        child: const Directionality(
          textDirection: TextDirection.ltr,
          child: TranslatedText.dynamic(
            'product.title',
            fallback: 'Product {id}',
            args: {'id': 42},
          ),
        ),
      ),
    );

    expect(find.text('Product 42'), findsOneWidget);

    controller.setOverride('product.title', 'Override {id}');
    await tester.pump();

    expect(find.text('Override 42'), findsOneWidget);
  });
}
