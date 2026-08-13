import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/route.dart';

void main() {
  test('generated parser round-trips web and app deep links', () {
    final webRoute = parseShoppingRoute(
      Uri.parse('https://shop.example/product/7?campaign=spring#reviews'),
    );
    final appRoute = parseShoppingRoute(
      Uri.parse('shopping://open/orders/ORDER%201%2F2?campaign=spring#receipt'),
    );

    expect(
      webRoute,
      isA<ShoppingProductDetailRoute>()
          .having((route) => route.productId, 'productId', 7)
          .having(
            (route) => route.location,
            'location',
            '/product/7?campaign=spring#reviews',
          ),
    );
    expect(
      appRoute,
      isA<ShoppingOrderDetailRoute>()
          .having((route) => route.orderId, 'orderId', 'ORDER 1/2')
          .having(
            (route) => route.location,
            'location',
            '/orders/ORDER%201%2F2?campaign=spring#receipt',
          ),
    );
  });

  test('known query parameters are typed while extras stay in the URL', () {
    final route = parseShoppingRoute(
      Uri.parse('/login?redirectPath=%2Fcheckout&coupon=SAVE10#auth'),
    );

    expect(
      route,
      isA<ShoppingLoginRoute>()
          .having((route) => route.redirectPath, 'redirectPath', '/checkout')
          .having(
            (route) => route.location,
            'location',
            '/login?redirectPath=%2Fcheckout&coupon=SAVE10#auth',
          ),
    );
  });

  test('unknown deep links resolve to the configured not-found route', () {
    final route = parseShoppingRoute(
      Uri.parse('https://shop.example/missing/path?campaign=spring#fallback'),
    );

    expect(
      route,
      isA<ShoppingNotFoundRoute>().having(
        (route) => route.path,
        'path',
        'https://shop.example/missing/path?campaign=spring#fallback',
      ),
    );
  });
}
