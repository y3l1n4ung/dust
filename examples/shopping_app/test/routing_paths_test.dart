import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/route.dart';

void main() {
  test('generated route locations round-trip encoded real app values', () {
    const orderId = 'ORDER 1/2';
    final order = ShoppingOrderDetailRoute(orderId: orderId);
    expect(order.location, '/orders/ORDER%201%2F2');

    final parsedOrder = parseShoppingRoute(Uri.parse(order.location));
    expect(
      parsedOrder,
      isA<ShoppingOrderDetailRoute>()
          .having((route) => route.orderId, 'orderId', orderId),
    );

    const redirectPath = '/orders/ORDER 1/2?tab=tracking';
    final login = ShoppingLoginRoute(redirectPath: redirectPath);
    expect(
      login.location,
      '/login?redirectPath=%2Forders%2FORDER+1%2F2%3Ftab%3Dtracking',
    );

    final parsedLogin = parseShoppingRoute(Uri.parse(login.location));
    expect(
      parsedLogin,
      isA<ShoppingLoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        redirectPath,
      ),
    );
  });

  test('typed deep links preserve undeclared query and fragment data', () {
    final route = parseShoppingRoute(
      Uri.parse('/product/7?campaign=spring&campaign=launch#reviews'),
    );

    expect(route, isA<ShoppingProductDetailRoute>());
    expect(
      route.location,
      '/product/7?campaign=spring&campaign=launch#reviews',
    );
  });

  test('shopping cart flow routes are typed and parseable', () {
    const cart = ShoppingCartRoute();
    const checkout = ShoppingCheckoutRoute();
    const confirmation = ShoppingOrderConfirmationRoute(orderId: 'ORDER 1/2');

    expect(cart.location, '/cart');
    expect(checkout.location, '/checkout');
    expect(confirmation.location, '/order-confirmation/ORDER%201%2F2');

    expect(
        parseShoppingRoute(Uri.parse(cart.location)), isA<ShoppingCartRoute>());
    expect(
      parseShoppingRoute(Uri.parse(checkout.location)),
      isA<ShoppingCheckoutRoute>(),
    );
    expect(
      parseShoppingRoute(Uri.parse(confirmation.location)),
      isA<ShoppingOrderConfirmationRoute>().having(
        (route) => route.orderId,
        'orderId',
        confirmation.orderId,
      ),
    );

    expect(shoppingRouteLocation(cart), cart.location);
    expect(shoppingRouteLocation(checkout), checkout.location);
    expect(shoppingRouteLocation(confirmation), confirmation.location);
  });

  test('invalid typed path values become not-found routes with source path',
      () {
    final route =
        parseShoppingRoute(Uri.parse('/product/not-an-int?from=deep-link'));

    expect(
      route,
      isA<ShoppingNotFoundRoute>().having(
        (route) => route.path,
        'path',
        '/product/not-an-int?from=deep-link',
      ),
    );
  });

  test('deep app routes restore the expected parent stack', () {
    final orderStack = restoreShoppingRouteStack(
      const ShoppingOrderDetailRoute(orderId: 'ORDER-9'),
    );
    expect(orderStack, [
      isA<ShoppingProductsRoute>(),
      isA<ShoppingOrdersRoute>(),
      isA<ShoppingOrderDetailRoute>().having(
        (route) => route.orderId,
        'orderId',
        'ORDER-9',
      ),
    ]);

    final supportStack =
        restoreShoppingRouteStack(const ShoppingSupportChatRoute());
    expect(
      supportStack,
      [isA<ShoppingProductsRoute>(), isA<ShoppingSupportChatRoute>()],
    );
  });
}
