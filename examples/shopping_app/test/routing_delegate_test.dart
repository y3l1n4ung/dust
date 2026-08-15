import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/route.dart';

import 'support/routing_support.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('shopping router redirects through real auth state safely', () async {
    final router = await shoppingRouter();
    final deepLink = parseShoppingRoute(
      Uri.parse('/orders/ORDER%201%2F2?campaign=spring#receipt'),
    ) as OrderDetailRoute;

    expect(
      router.redirect(const CheckoutRoute()),
      isA<LoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/checkout',
      ),
    );
    expect(
      router.redirect(deepLink),
      isA<LoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        deepLink.location,
      ),
    );
    expect(router.redirect(const ProductsRoute()), isNull);

    await router.auth.login('dust', 'password');

    expect(
      router.redirect(LoginRoute(redirectPath: deepLink.location)),
      isA<OrderDetailRoute>().having(
        (route) => route.orderId,
        'orderId',
        deepLink.orderId,
      ),
    );

    final unsafeAuthRoutes = <ShoppingRoute>[
      const LoginRoute(),
      const LoginRoute(redirectPath: ''),
      const LoginRoute(redirectPath: 'https://evil.test/a'),
      const RegisterRoute(redirectPath: '//evil.test/a'),
      const RegisterRoute(redirectPath: '/missing'),
      const LoginRoute(redirectPath: '/404?path=/orders/ORDER-1'),
    ];
    for (final route in unsafeAuthRoutes) {
      expect(router.redirect(route), isA<ProductsRoute>());
    }
  });

  test('shopping router waits while auth state is unresolved', () async {
    final router = await shoppingRouter();

    setAuthInitial(router);
    expect(router.redirect(const CheckoutRoute()), isNull);

    setAuthLoading(router);
    expect(router.redirect(const CheckoutRoute()), isNull);
  });

  test('router redirect and guards compose through runtime navigation',
      () async {
    final router = await shoppingRouter();
    final delegate =
        router.config.routerDelegate as GeneratedRouterDelegate<ShoppingRoute>;
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const AdminRoute());
    expect(
      delegate.currentConfiguration,
      isA<LoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/admin',
      ),
    );
    expect(
      delegate.stack,
      [isA<ProductsRoute>(), isA<LoginRoute>()],
    );

    authenticate(router, 'dust');
    await delegate.debugWaitForScheduledRefresh();
    expect(delegate.currentConfiguration, isA<ProductsRoute>());

    await delegate.setNewRoutePath(const StaffRoute());
    expect(delegate.currentConfiguration, isA<ProductsRoute>());
    expect(delegate.stack, [isA<ProductsRoute>()]);

    authenticate(router, 'manager');
    await delegate.debugWaitForScheduledRefresh();
    await delegate.setNewRoutePath(const StaffRoute());
    expect(delegate.currentConfiguration, isA<StaffRoute>());
    expect(
      delegate.stack,
      [isA<ProductsRoute>(), isA<StaffRoute>()],
    );

    await delegate.setNewRoutePath(const AdminRoute());
    expect(delegate.currentConfiguration, isA<ProductsRoute>());

    authenticate(router, 'admin');
    await delegate.debugWaitForScheduledRefresh();
    await delegate.setNewRoutePath(const AdminRoute());
    expect(delegate.currentConfiguration, isA<AdminRoute>());
    expect(
      delegate.stack,
      [isA<ProductsRoute>(), isA<AdminRoute>()],
    );

    expireSession(router);
    await delegate.debugWaitForScheduledRefresh();
    expect(
      delegate.currentConfiguration,
      isA<LoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/admin',
      ),
    );
    expect(
      delegate.stack,
      [isA<ProductsRoute>(), isA<LoginRoute>()],
    );

    authenticate(router, 'dust');
    await delegate.debugWaitForScheduledRefresh();
    expect(delegate.currentConfiguration, isA<ProductsRoute>());
  });

  test('router revalidates protected routes exposed by pop', () async {
    final router = await shoppingRouter();
    final delegate =
        router.config.routerDelegate as GeneratedRouterDelegate<ShoppingRoute>;
    await delegate.debugWaitForScheduledRefresh();

    authenticate(router, 'admin');
    await delegate.debugWaitForScheduledRefresh();
    await delegate.setNewRoutePath(const AdminRoute());
    expect(
      delegate.stack,
      [isA<ProductsRoute>(), isA<AdminRoute>()],
    );

    final supportResult = delegate.push<bool>(const SupportChatRoute());
    await Future<void>.delayed(Duration.zero);
    expect(delegate.stack, [
      isA<ProductsRoute>(),
      isA<AdminRoute>(),
      isA<SupportChatRoute>(),
    ]);

    expireSession(router);
    await delegate.popRoute();
    await Future<void>.delayed(Duration.zero);

    expect(
      delegate.currentConfiguration,
      isA<LoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/admin',
      ),
    );
    expect(
      delegate.stack,
      [isA<ProductsRoute>(), isA<LoginRoute>()],
    );
    await expectLater(supportResult, completion(isNull));
  });
}
