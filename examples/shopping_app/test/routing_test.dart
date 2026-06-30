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
    final order = OrderDetailRoute(orderId: orderId);
    expect(order.location, isNot(contains(' ')));

    final parsedOrder = parseAppRoute(Uri.parse(order.location));
    expect(
      parsedOrder,
      isA<OrderDetailRoute>()
          .having((route) => route.orderId, 'orderId', orderId),
    );

    const redirectPath = '/orders/ORDER 1/2?tab=tracking';
    final login = LoginRoute(redirectPath: redirectPath);
    expect(login.location, startsWith('/login?redirectPath='));
    expect(login.location, isNot(contains(' ')));

    final parsedLogin = parseAppRoute(Uri.parse(login.location));
    expect(
      parsedLogin,
      isA<LoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        redirectPath,
      ),
    );
  });

  test('invalid typed path values become not-found routes with source path',
      () {
    final route =
        parseAppRoute(Uri.parse('/product/not-an-int?from=deep-link'));

    expect(
      route,
      isA<NotFoundRoute>().having(
        (route) => route.path,
        'path',
        '/product/not-an-int?from=deep-link',
      ),
    );
  });

  test('real app public and protected route contract is explicit', () async {
    final router = await _shoppingRouter();

    final publicRoutes = <AppRoutePath<void>>[
      const ProductsRoute(),
      const CartRoute(),
      const LoginRoute(redirectPath: '/checkout'),
      const ProductDetailRoute(productId: 1),
      const SupportChatRoute(),
      const WishlistRoute(),
    ];
    for (final route in publicRoutes) {
      expect(routeRequiresAuth(route), isFalse, reason: '${route.runtimeType}');
    }

    final protectedRoutes = <AppRoutePath<void>>[
      const AdminRoute(),
      const CheckoutRoute(),
      const OrdersRoute(),
      const OrderDetailRoute(orderId: 'ORDER-1'),
      const ProfileRoute(),
      const StaffRoute(),
    ];
    for (final route in protectedRoutes) {
      expect(routeRequiresAuth(route), isTrue, reason: '${route.runtimeType}');
    }

    expect(routeGuards(const CheckoutRoute(), router), isEmpty);
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
    const staffRoute = StaffRoute();
    const adminRoute = AdminRoute();

    final staffGuard = routeGuards(staffRoute, router).single as StaffGuard;
    final adminGuard = routeGuards(adminRoute, router).single as AdminGuard;
    expect(staffGuard.auth, same(router.auth));
    expect(adminGuard.auth, same(router.auth));

    expect(
      staffGuard.canActivate(staffRoute),
      isA<LoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        staffRoute.location,
      ),
    );

    _authenticate(router, 'dust');
    expect(staffGuard.canActivate(staffRoute), isA<ProductsRoute>());
    expect(adminGuard.canActivate(adminRoute), isA<ProductsRoute>());

    _authenticate(router, 'manager');
    expect(staffGuard.canActivate(staffRoute), isNull);
    expect(adminGuard.canActivate(adminRoute), isA<ProductsRoute>());

    _authenticate(router, 'admin');
    expect(staffGuard.canActivate(staffRoute), isNull);
    expect(adminGuard.canActivate(adminRoute), isNull);

    expect(routeGuards(const ProductsRoute(), router), isEmpty);
  });

  test('deep app routes restore the expected parent stack', () {
    final orderStack = restoreAppRouteStack(
      const OrderDetailRoute(orderId: 'ORDER-9'),
    );
    expect(orderStack, [
      isA<ProductsRoute>(),
      isA<OrdersRoute>(),
      isA<OrderDetailRoute>().having(
        (route) => route.orderId,
        'orderId',
        'ORDER-9',
      ),
    ]);

    final supportStack = restoreAppRouteStack(const SupportChatRoute());
    expect(supportStack, [isA<ProductsRoute>(), isA<SupportChatRoute>()]);
  });

  test('shopping router redirects through real auth state safely', () async {
    final router = await _shoppingRouter();
    const deepLink = OrderDetailRoute(orderId: 'ORDER 1/2');

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

    final unsafeAuthRoutes = <AppRoutePath<void>>[
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
    final router = await _shoppingRouter();

    router.auth.value = const AuthState(status: AuthStatus.initial);
    expect(router.redirect(const CheckoutRoute()), isNull);

    router.auth.value = const AuthState(status: AuthStatus.loading);
    expect(router.redirect(const CheckoutRoute()), isNull);
  });

  test('router redirect and guards compose through runtime navigation',
      () async {
    final router = await _shoppingRouter();
    final delegate =
        router.config.routerDelegate as GeneratedRouterDelegate<AppRoutePath>;
    await _settleScheduledRouterRefresh();

    await delegate.setNewRoutePath(const AdminRoute());
    expect(
      delegate.currentConfiguration,
      isA<LoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/admin',
      ),
    );
    expect(delegate.stack, [isA<ProductsRoute>(), isA<LoginRoute>()]);

    _authenticate(router, 'dust');
    await _settleScheduledRouterRefresh();
    expect(delegate.currentConfiguration, isA<ProductsRoute>());

    await delegate.setNewRoutePath(const StaffRoute());
    expect(delegate.currentConfiguration, isA<ProductsRoute>());
    expect(delegate.stack, [isA<ProductsRoute>()]);

    _authenticate(router, 'manager');
    await _settleScheduledRouterRefresh();
    await delegate.setNewRoutePath(const StaffRoute());
    expect(delegate.currentConfiguration, isA<StaffRoute>());
    expect(delegate.stack, [isA<ProductsRoute>(), isA<StaffRoute>()]);

    await delegate.setNewRoutePath(const AdminRoute());
    expect(delegate.currentConfiguration, isA<ProductsRoute>());

    _authenticate(router, 'admin');
    await _settleScheduledRouterRefresh();
    await delegate.setNewRoutePath(const AdminRoute());
    expect(delegate.currentConfiguration, isA<AdminRoute>());
    expect(delegate.stack, [isA<ProductsRoute>(), isA<AdminRoute>()]);

    _expireSession(router);
    await _settleScheduledRouterRefresh();
    expect(
      delegate.currentConfiguration,
      isA<LoginRoute>().having(
        (route) => route.redirectPath,
        'redirectPath',
        '/admin',
      ),
    );
    expect(delegate.stack, [isA<ProductsRoute>(), isA<LoginRoute>()]);

    _authenticate(router, 'dust');
    await _settleScheduledRouterRefresh();
    expect(delegate.currentConfiguration, isA<ProductsRoute>());
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

Future<void> _settleScheduledRouterRefresh() async {
  // Auth changes schedule a router refresh microtask; that refresh then awaits
  // the guard chain before committing the replacement route.
  await Future<void>.delayed(Duration.zero);
  await Future<void>.delayed(Duration.zero);
}
