import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/widgets.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shopping_app/core/services/storage_service.dart';
import 'package:shopping_app/features/checkout/models/checkout_quote.dart';
import 'package:shopping_app/features/products/models/product.dart';
import 'package:shopping_app/features/support/models/chat_message.dart';
import 'package:shopping_app/i18n/app_i18n.g.dart';
import 'package:shopping_app/main.dart';
import 'package:shopping_app/route.dart';

import 'support/fake_shopping_repository.dart';

void main() {
  testWidgets('Shopping app loads with generated app scope and router', (
    tester,
  ) async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();

    await tester.pumpWidget(
      AppI18n(
        child: ShoppingApp(
          storage: StorageService(prefs),
          repository: FakeShoppingRepository(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Shop'), findsOneWidget);
    expect(find.text('EN'), findsOneWidget);
    expect(find.text('Dust Backpack'), findsOneWidget);
    expect(find.text('Bags'), findsWidgets);
    expect(find.text(r'$42.00'), findsOneWidget);
    expect(find.text('4.8 (12)'), findsOneWidget);

    await tester.tap(find.text('EN'));
    await tester.pumpAndSettle();

    expect(find.text('ဆိုင်'), findsOneWidget);
    expect(find.text('MY'), findsOneWidget);
    expect(find.text('Dust Backpack'), findsOneWidget);
    expect(find.text('အိတ်များ'), findsWidgets);
    expect(find.text(r'US$ 42.00'), findsOneWidget);
    expect(find.text('4.8 (12)'), findsOneWidget);
  });

  testWidgets('Shopping app replaces scoped repository dependencies', (
    tester,
  ) async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final storage = StorageService(prefs);

    Widget build(FakeShoppingRepository repository) {
      return AppI18n(
        child: ShoppingApp(
          storage: storage,
          repository: repository,
        ),
      );
    }

    await tester.pumpWidget(
      build(
        FakeShoppingRepository(
          products: const [
            Product(
              id: 1,
              title: 'First Scoped Backpack',
              price: 42,
              description: 'First repository product.',
              category: 'bags',
              image: 'https://example.com/first.png',
              rating: Rating(rate: 4.8, count: 12),
            ),
          ],
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('First Scoped Backpack'), findsOneWidget);
    expect(find.text('Second Scoped Backpack'), findsNothing);

    await tester.pumpWidget(
      build(
        FakeShoppingRepository(
          products: const [
            Product(
              id: 2,
              title: 'Second Scoped Backpack',
              price: 64,
              description: 'Second repository product.',
              category: 'bags',
              image: 'https://example.com/second.png',
              rating: Rating(rate: 4.9, count: 18),
            ),
          ],
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Second Scoped Backpack'), findsOneWidget);
    expect(find.text('First Scoped Backpack'), findsNothing);
  });

  test('generated routes include new shopping showcase destinations', () {
    expect(parseAppRoute(Uri.parse('/wishlist')), isA<WishlistRoute>());
    expect(parseAppRoute(Uri.parse('/demo-carts')), isA<DemoCartsRoute>());
    expect(parseAppRoute(Uri.parse('/support/chat')), isA<SupportChatRoute>());
    expect(
      parseAppRoute(Uri.parse('/orders/ORDER-1')),
      isA<OrderDetailRoute>(),
    );
  });

  test(
    'fake repository supports quote, tracking, and chat demo contracts',
    () async {
      final repository = FakeShoppingRepository();

      final quote = await repository.quoteCheckout(
        const CheckoutQuoteRequest(subtotal: 100, couponCode: 'DUST10'),
      );
      expect(quote.total, lessThan(115));
      expect(quote.appliedCoupon, 'DUST10');

      final tracking = await repository.getOrderTracking('ORDER-1');
      expect(tracking, isNotEmpty);

      final socket = repository.openChatSocket();
      final responseFuture = socket.responses.first;
      socket.send(const ChatRequest(message: 'coupon help', history: []));
      final chat = await responseFuture;
      await socket.close();

      expect(chat.message.role, ChatRole.assistant);
    },
  );
}
