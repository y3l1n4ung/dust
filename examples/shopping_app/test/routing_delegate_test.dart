import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/route.dart';

import 'support/routing_support.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('shopping router redirects through real auth state safely', () async {
    final router = await shoppingRouter();
    final deepLink = parseShoppingRoute(
      Uri.parse('/orders/ORDER%201%2F2?campaign=spring#receipt'),
    ) as ShoppingOrderDetailRoute;

    expect(
      router.redirect(const ShoppingCheckoutRoute()),
      isA<ShoppingLoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/checkout',
      ),
    );
    expect(
      router.redirect(deepLink),
      isA<ShoppingLoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        deepLink.location,
      ),
    );
    expect(router.redirect(const ShoppingProductsRoute()), isNull);

    await router.auth.login('dust', 'password');

    expect(
      router.redirect(ShoppingLoginRoute(redirectPath: deepLink.location)),
      isA<ShoppingOrderDetailRoute>().having(
        (route) => route.orderId,
        'orderId',
        deepLink.orderId,
      ),
    );

    final unsafeAuthRoutes = <ShoppingRoutePath>[
      const ShoppingLoginRoute(),
      const ShoppingLoginRoute(redirectPath: ''),
      const ShoppingLoginRoute(redirectPath: 'https://evil.test/a'),
      const ShoppingRegisterRoute(redirectPath: '//evil.test/a'),
      const ShoppingRegisterRoute(redirectPath: '/missing'),
      const ShoppingLoginRoute(redirectPath: '/404?path=/orders/ORDER-1'),
    ];
    for (final route in unsafeAuthRoutes) {
      expect(router.redirect(route), isA<ShoppingProductsRoute>());
    }
  });

  test('shopping router waits while auth state is unresolved', () async {
    final router = await shoppingRouter();

    setAuthInitial(router);
    expect(router.redirect(const ShoppingCheckoutRoute()), isNull);

    setAuthLoading(router);
    expect(router.redirect(const ShoppingCheckoutRoute()), isNull);
  });

  test('router redirect and guards compose through runtime navigation',
      () async {
    final router = await shoppingRouter();
    final delegate = router.config.routerDelegate
        as GeneratedRouterDelegate<ShoppingRoutePath>;
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const ShoppingAdminRoute());
    expect(
      delegate.currentConfiguration,
      isA<ShoppingLoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/admin',
      ),
    );
    expect(
      delegate.stack,
      [isA<ShoppingProductsRoute>(), isA<ShoppingLoginRoute>()],
    );

    authenticate(router, 'dust');
    await delegate.debugWaitForScheduledRefresh();
    expect(delegate.currentConfiguration, isA<ShoppingProductsRoute>());

    await delegate.setNewRoutePath(const ShoppingStaffRoute());
    expect(delegate.currentConfiguration, isA<ShoppingProductsRoute>());
    expect(delegate.stack, [isA<ShoppingProductsRoute>()]);

    authenticate(router, 'manager');
    await delegate.debugWaitForScheduledRefresh();
    await delegate.setNewRoutePath(const ShoppingStaffRoute());
    expect(delegate.currentConfiguration, isA<ShoppingStaffRoute>());
    expect(
      delegate.stack,
      [isA<ShoppingProductsRoute>(), isA<ShoppingStaffRoute>()],
    );

    await delegate.setNewRoutePath(const ShoppingAdminRoute());
    expect(delegate.currentConfiguration, isA<ShoppingProductsRoute>());

    authenticate(router, 'admin');
    await delegate.debugWaitForScheduledRefresh();
    await delegate.setNewRoutePath(const ShoppingAdminRoute());
    expect(delegate.currentConfiguration, isA<ShoppingAdminRoute>());
    expect(
      delegate.stack,
      [isA<ShoppingProductsRoute>(), isA<ShoppingAdminRoute>()],
    );

    expireSession(router);
    await delegate.debugWaitForScheduledRefresh();
    expect(
      delegate.currentConfiguration,
      isA<ShoppingLoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/admin',
      ),
    );
    expect(
      delegate.stack,
      [isA<ShoppingProductsRoute>(), isA<ShoppingLoginRoute>()],
    );

    authenticate(router, 'dust');
    await delegate.debugWaitForScheduledRefresh();
    expect(delegate.currentConfiguration, isA<ShoppingProductsRoute>());
  });

  test('router revalidates protected routes exposed by pop', () async {
    final router = await shoppingRouter();
    final delegate = router.config.routerDelegate
        as GeneratedRouterDelegate<ShoppingRoutePath>;
    await delegate.debugWaitForScheduledRefresh();

    authenticate(router, 'admin');
    await delegate.debugWaitForScheduledRefresh();
    await delegate.setNewRoutePath(const ShoppingAdminRoute());
    expect(
      delegate.stack,
      [isA<ShoppingProductsRoute>(), isA<ShoppingAdminRoute>()],
    );

    final supportResult = delegate.push<bool>(const ShoppingSupportChatRoute());
    await Future<void>.delayed(Duration.zero);
    expect(delegate.stack, [
      isA<ShoppingProductsRoute>(),
      isA<ShoppingAdminRoute>(),
      isA<ShoppingSupportChatRoute>(),
    ]);

    expireSession(router);
    await delegate.popRoute();
    await Future<void>.delayed(Duration.zero);

    expect(
      delegate.currentConfiguration,
      isA<ShoppingLoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/admin',
      ),
    );
    expect(
      delegate.stack,
      [isA<ShoppingProductsRoute>(), isA<ShoppingLoginRoute>()],
    );
    await expectLater(supportResult, completion(isNull));
  });
}
