import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shopping_app/core/services/storage_service.dart';
import 'package:shopping_app/features/auth/models/auth_state.dart';
import 'package:shopping_app/features/auth/models/user.dart';
import 'package:shopping_app/features/auth/view_models/auth_view_model.dart';
import 'package:shopping_app/route.dart';

import 'support/fake_shopping_repository.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

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
        route.location, '/product/7?campaign=spring&campaign=launch#reviews');
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

  test('real app public and protected route contract is explicit', () async {
    final router = await _shoppingRouter();

    final publicRoutes = <ShoppingRoutePath>[
      const ShoppingProductsRoute(),
      const ShoppingCartRoute(),
      const ShoppingLoginRoute(redirectPath: '/checkout'),
      const ShoppingProductDetailRoute(productId: 1),
      const ShoppingSupportChatRoute(),
      const ShoppingWishlistRoute(),
    ];
    const supportRoute = ShoppingSupportChatRoute();
    expect(supportRoute, isA<ShoppingRoutePath<bool>>());
    for (final route in publicRoutes) {
      expect(shoppingRouteRequiresAuth(route), isFalse,
          reason: '${route.runtimeType}');
    }

    final protectedRoutes = <ShoppingRoutePath>[
      const ShoppingAdminRoute(),
      const ShoppingCheckoutRoute(),
      const ShoppingOrdersRoute(),
      const ShoppingOrderDetailRoute(orderId: 'ORDER-1'),
      const ShoppingProfileRoute(),
      const ShoppingStaffRoute(),
    ];
    for (final route in protectedRoutes) {
      expect(shoppingRouteRequiresAuth(route), isTrue,
          reason: '${route.runtimeType}');
    }

    expect(shoppingRouteGuards(const ShoppingCheckoutRoute(), router), isEmpty);
  });

  test('shopping access levels map real app users', () {
    expect(shoppingAccessLevel(null), ShoppingAccessLevel.guest);
    expect(shoppingAccessLevel(_user('dust')), ShoppingAccessLevel.customer);
    expect(shoppingAccessLevel(_user('staff')), ShoppingAccessLevel.staff);
    expect(shoppingAccessLevel(_user('manager')), ShoppingAccessLevel.staff);
    expect(shoppingAccessLevel(_user('admin')), ShoppingAccessLevel.admin);
  });

  test('staff and admin guards use injected auth access levels', () async {
    final router = await _shoppingRouter();
    const staffRoute = ShoppingStaffRoute();
    const adminRoute = ShoppingAdminRoute();

    final staffGuard =
        shoppingRouteGuards(staffRoute, router).single as StaffGuard;
    final adminGuard =
        shoppingRouteGuards(adminRoute, router).single as AdminGuard;
    expect(staffGuard.auth, same(router.auth));
    expect(adminGuard.auth, same(router.auth));

    expect(
      staffGuard.canActivate(staffRoute),
      isA<ShoppingLoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        staffRoute.location,
      ),
    );

    _authenticate(router, 'dust');
    expect(staffGuard.canActivate(staffRoute), isA<ShoppingProductsRoute>());
    expect(adminGuard.canActivate(adminRoute), isA<ShoppingProductsRoute>());

    _authenticate(router, 'manager');
    expect(staffGuard.canActivate(staffRoute), isNull);
    expect(adminGuard.canActivate(adminRoute), isA<ShoppingProductsRoute>());

    _authenticate(router, 'admin');
    expect(staffGuard.canActivate(staffRoute), isNull);
    expect(adminGuard.canActivate(adminRoute), isNull);

    expect(shoppingRouteGuards(const ShoppingProductsRoute(), router), isEmpty);
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
    expect(supportStack,
        [isA<ShoppingProductsRoute>(), isA<ShoppingSupportChatRoute>()]);
  });

  test('shopping router redirects through real auth state safely', () async {
    final router = await _shoppingRouter();
    final deepLink = parseShoppingRoute(
            Uri.parse('/orders/ORDER%201%2F2?campaign=spring#receipt'))
        as ShoppingOrderDetailRoute;

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
    final router = await _shoppingRouter();

    router.auth.value = const AuthState(status: AuthStatus.initial);
    expect(router.redirect(const ShoppingCheckoutRoute()), isNull);

    router.auth.value = const AuthState(status: AuthStatus.loading);
    expect(router.redirect(const ShoppingCheckoutRoute()), isNull);
  });

  test('router redirect and guards compose through runtime navigation',
      () async {
    final router = await _shoppingRouter();
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
    expect(delegate.stack,
        [isA<ShoppingProductsRoute>(), isA<ShoppingLoginRoute>()]);

    _authenticate(router, 'dust');
    await delegate.debugWaitForScheduledRefresh();
    expect(delegate.currentConfiguration, isA<ShoppingProductsRoute>());

    await delegate.setNewRoutePath(const ShoppingStaffRoute());
    expect(delegate.currentConfiguration, isA<ShoppingProductsRoute>());
    expect(delegate.stack, [isA<ShoppingProductsRoute>()]);

    _authenticate(router, 'manager');
    await delegate.debugWaitForScheduledRefresh();
    await delegate.setNewRoutePath(const ShoppingStaffRoute());
    expect(delegate.currentConfiguration, isA<ShoppingStaffRoute>());
    expect(delegate.stack,
        [isA<ShoppingProductsRoute>(), isA<ShoppingStaffRoute>()]);

    await delegate.setNewRoutePath(const ShoppingAdminRoute());
    expect(delegate.currentConfiguration, isA<ShoppingProductsRoute>());

    _authenticate(router, 'admin');
    await delegate.debugWaitForScheduledRefresh();
    await delegate.setNewRoutePath(const ShoppingAdminRoute());
    expect(delegate.currentConfiguration, isA<ShoppingAdminRoute>());
    expect(delegate.stack,
        [isA<ShoppingProductsRoute>(), isA<ShoppingAdminRoute>()]);

    _expireSession(router);
    await delegate.debugWaitForScheduledRefresh();
    expect(
      delegate.currentConfiguration,
      isA<ShoppingLoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/admin',
      ),
    );
    expect(delegate.stack,
        [isA<ShoppingProductsRoute>(), isA<ShoppingLoginRoute>()]);

    _authenticate(router, 'dust');
    await delegate.debugWaitForScheduledRefresh();
    expect(delegate.currentConfiguration, isA<ShoppingProductsRoute>());
  });

  test('router revalidates protected routes exposed by pop', () async {
    final router = await _shoppingRouter();
    final delegate = router.config.routerDelegate
        as GeneratedRouterDelegate<ShoppingRoutePath>;
    await delegate.debugWaitForScheduledRefresh();

    _authenticate(router, 'admin');
    await delegate.debugWaitForScheduledRefresh();
    await delegate.setNewRoutePath(const ShoppingAdminRoute());
    expect(delegate.stack,
        [isA<ShoppingProductsRoute>(), isA<ShoppingAdminRoute>()]);

    final supportResult = delegate.push<bool>(const ShoppingSupportChatRoute());
    await Future<void>.delayed(Duration.zero);
    expect(delegate.stack, [
      isA<ShoppingProductsRoute>(),
      isA<ShoppingAdminRoute>(),
      isA<ShoppingSupportChatRoute>(),
    ]);

    _expireSession(router);
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
    expect(delegate.stack,
        [isA<ShoppingProductsRoute>(), isA<ShoppingLoginRoute>()]);
    await expectLater(supportResult, completion(isNull));
  });
}

Future<ShoppingRouter> _shoppingRouter() async {
  SharedPreferences.setMockInitialValues({});
  final prefs = await SharedPreferences.getInstance();
  final auth = AuthViewModel(
    AuthViewModelArgs(
      repository: FakeShoppingRepository(),
      storage: StorageService(prefs),
    ),
  );
  addTearDown(auth.dispose);
  return ShoppingRouter(auth: auth);
}

void _authenticate(ShoppingRouter router, String username) {
  router.auth.value = AuthState(
    status: AuthStatus.authenticated,
    token: 'token-$username',
    user: _user(username),
  );
}

void _expireSession(ShoppingRouter router) {
  router.auth.value = const AuthState(status: AuthStatus.unauthenticated);
}

User _user(String username) {
  return User(
    id: username.hashCode,
    email: '$username@example.com',
    username: username,
    name: Name(firstname: username, lastname: 'User'),
    phone: '555-0100',
  );
}
