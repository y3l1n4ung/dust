import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shopping_app/core/services/storage_service.dart';
import 'package:shopping_app/features/checkout/models/checkout_quote.dart';
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
