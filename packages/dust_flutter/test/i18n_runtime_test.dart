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

  test('controller handles override mutations', () {
    final controller = I18nController(
      config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
      bundles: const [
        I18nBundle(
          locale: 'en',
          namespace: 'home',
          messages: {'title': 'Home'},
        ),
      ],
    );
    var changes = 0;
    controller.addListener(() => changes += 1);

    expect(controller.locale, 'en');
    controller.setOverride('home_title', 'Override');
    expect(controller.translate('home_title'), 'Override');
    expect(changes, 1);

    controller.setOverride('missing', null);
    expect(changes, 1);

    controller.setOverride('home_title', null);
    expect(controller.translate('home_title'), 'Home');
    expect(changes, 2);

    controller.setOverrides({'home_title': 'Bulk'});
    expect(controller.translate('home_title'), 'Bulk');
    expect(changes, 3);

    controller.setOverrides({'home_title': 'Bulk'});
    expect(changes, 3);
  });

  test('controller uses the longest loaded namespace prefix', () {
    final controller = I18nController(
      config: const I18nConfig(
        locales: ['en'],
        fallbackLocale: 'en',
      ),
      bundles: const [
        I18nBundle(
          locale: 'en',
          namespace: 'shop',
          messages: {'product_price': 'Shop price'},
        ),
        I18nBundle(
          locale: 'en',
          namespace: 'shop_product',
          messages: {'price': 'Product price'},
        ),
      ],
    );

    expect(controller.translate('shop_product_price'), 'Product price');
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
              const TranslatedText('home_title'),
              Builder(
                builder: (context) {
                  return Text(
                    context.tr('home_cta', args: {'name': 'Dust'}),
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
      config: const I18nConfig(
        locales: ['en'],
        fallbackLocale: 'en',
      ),
    );

    await tester.pumpWidget(
      I18nScope(
        controller: controller,
        child: const Directionality(
          textDirection: TextDirection.ltr,
          child: TranslatedText.dynamic(
            'product_title',
            fallback: 'Product {id}',
            args: {'id': 42},
          ),
        ),
      ),
    );

    expect(find.text('Product 42'), findsOneWidget);

    controller.setOverride('product_title', 'Override {id}');
    await tester.pump();

    expect(find.text('Override 42'), findsOneWidget);
  });

  testWidgets('I18nScope.of throws when missing', (tester) async {
    BuildContext? capturedContext;

    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: Builder(
          builder: (context) {
            capturedContext = context;
            return const SizedBox();
          },
        ),
      ),
    );

    expect(
      () => I18nScope.of(capturedContext!),
      throwsA(isA<FlutterError>()),
    );
  });

  testWidgets('TranslatedText forwards Text configuration', (tester) async {
    const style = TextStyle(fontSize: 18, fontWeight: FontWeight.w600);
    const strutStyle = StrutStyle(fontSize: 20);
    const locale = Locale('my');
    const heightBehavior = TextHeightBehavior(
      applyHeightToFirstAscent: false,
      applyHeightToLastDescent: false,
    );
    const selectionColor = Color(0xFF336699);
    const textScaler = TextScaler.linear(1.2);
    final controller = I18nController(
      config: const I18nConfig(
        locales: ['en'],
        fallbackLocale: 'en',
      ),
      bundles: const [
        I18nBundle(
          locale: 'en',
          namespace: 'home',
          messages: {'title': 'Home'},
        ),
      ],
    );

    await tester.pumpWidget(
      I18nScope(
        controller: controller,
        child: const Directionality(
          textDirection: TextDirection.ltr,
          child: TranslatedText(
            'home_title',
            style: style,
            strutStyle: strutStyle,
            textAlign: TextAlign.center,
            textDirection: TextDirection.rtl,
            locale: locale,
            softWrap: false,
            overflow: TextOverflow.ellipsis,
            textScaler: textScaler,
            maxLines: 1,
            semanticsLabel: 'Home label',
            semanticsIdentifier: 'home-title',
            textWidthBasis: TextWidthBasis.longestLine,
            textHeightBehavior: heightBehavior,
            selectionColor: selectionColor,
          ),
        ),
      ),
    );

    final text = tester.widget<Text>(find.text('Home'));
    expect(text.style, style);
    expect(text.strutStyle, strutStyle);
    expect(text.textAlign, TextAlign.center);
    expect(text.textDirection, TextDirection.rtl);
    expect(text.locale, locale);
    expect(text.softWrap, isFalse);
    expect(text.overflow, TextOverflow.ellipsis);
    expect(text.textScaler, textScaler);
    expect(text.maxLines, 1);
    expect(text.semanticsLabel, 'Home label');
    expect(text.semanticsIdentifier, 'home-title');
    expect(text.textWidthBasis, TextWidthBasis.longestLine);
    expect(text.textHeightBehavior, heightBehavior);
    expect(text.selectionColor, selectionColor);
  });

  testWidgets('TranslatedText converts legacy text scale factor', (
    tester,
  ) async {
    final controller = I18nController(
      config: const I18nConfig(locales: ['en'], fallbackLocale: 'en'),
      bundles: const [
        I18nBundle(
          locale: 'en',
          namespace: 'home',
          messages: {'title': 'Home'},
        ),
      ],
    );

    await tester.pumpWidget(
      I18nScope(
        controller: controller,
        child: const Directionality(
          textDirection: TextDirection.ltr,
          child: TranslatedText('home_title', textScaleFactor: 1.4),
        ),
      ),
    );

    final text = tester.widget<Text>(find.text('Home'));
    expect(text.textScaler, const TextScaler.linear(1.4));
  });
}
