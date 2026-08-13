import 'package:flutter/widgets.dart' show RouteInformation;
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

  test('absolute deep links preserve duplicate unknown query values', () {
    final route = parseShoppingRoute(
      Uri.parse(
        'https://shop.example/product/7?campaign=spring&campaign=launch'
        '&redirect=%2Fcheckout#reviews',
      ),
    );

    expect(
      route,
      isA<ShoppingProductDetailRoute>()
          .having((route) => route.productId, 'productId', 7)
          .having(
            (route) => route.location,
            'location',
            '/product/7?campaign=spring&campaign=launch'
                '&redirect=%2Fcheckout#reviews',
          ),
    );
  });

  test('trailing slash deep links use the configured not-found route', () {
    final route = parseShoppingRoute(
      Uri.parse('https://shop.example/product/7/?campaign=spring#reviews'),
    );

    expect(
      route,
      isA<ShoppingNotFoundRoute>().having(
        (route) => route.path,
        'path',
        'https://shop.example/product/7/?campaign=spring#reviews',
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

  test('generated parser applies app route-information override first',
      () async {
    final state = Object();
    final router = _ShoppingRouteInformationRouter();
    final parser = GeneratedRouteInformationParser<ShoppingRoutePath>(
      router: router,
      parseRoute: parseShoppingRoute,
      routeLocation: shoppingRouteLocation,
    );

    final prefixedRoute = await parser.parseRouteInformation(
      RouteInformation(
        uri: Uri.parse(
          'https://shop.example/app/product/7?campaign=spring#reviews',
        ),
        state: state,
      ),
    );
    final unsafeHostRoute = await parser.parseRouteInformation(
      RouteInformation(
        uri: Uri.parse('https://evil.test/app/product/7'),
        state: state,
      ),
    );

    expect(
      prefixedRoute,
      isA<ShoppingProductDetailRoute>()
          .having((route) => route.productId, 'productId', 7)
          .having(
            (route) => route.location,
            'location',
            '/product/7?campaign=spring#reviews',
          ),
    );
    expect(
      unsafeHostRoute,
      isA<ShoppingNotFoundRoute>().having(
        (route) => route.path,
        'path',
        'https://evil.test/app/product/7',
      ),
    );
    expect(router.seenStates.length, 2);
    expect(router.seenStates.every((seen) => identical(seen, state)), isTrue);
  });
}

final class _ShoppingRouteInformationRouter
    extends RouterBase<ShoppingRoutePath> {
  final seenStates = <Object?>[];

  @override
  RouteInformation parseRouteInformation(RouteInformation information) {
    seenStates.add(information.state);
    final uri = information.uri;
    if (uri.hasAuthority && uri.host != 'shop.example') {
      return RouteInformation(
        uri: Uri(path: '/404', queryParameters: {'path': uri.toString()}),
        state: information.state,
      );
    }
    if (!uri.path.startsWith('/app/')) return information;
    return RouteInformation(
      uri: uri.replace(path: uri.path.substring('/app'.length)),
      state: information.state,
    );
  }
}
