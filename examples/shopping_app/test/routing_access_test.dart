import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/route.dart';

import 'support/routing_support.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('real app public and protected route contract is explicit', () async {
    final router = await shoppingRouter();

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
      expect(
        shoppingRouteRequiresAuth(route),
        isFalse,
        reason: '${route.runtimeType}',
      );
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
      expect(
        shoppingRouteRequiresAuth(route),
        isTrue,
        reason: '${route.runtimeType}',
      );
    }

    expect(shoppingRouteGuards(const ShoppingCheckoutRoute(), router), isEmpty);
  });

  test('shopping access levels map real app users', () {
    expect(shoppingAccessLevel(null), ShoppingAccessLevel.guest);
    expect(shoppingAccessLevel(user('dust')), ShoppingAccessLevel.customer);
    expect(shoppingAccessLevel(user('staff')), ShoppingAccessLevel.staff);
    expect(shoppingAccessLevel(user('manager')), ShoppingAccessLevel.staff);
    expect(shoppingAccessLevel(user('admin')), ShoppingAccessLevel.admin);
  });

  test('staff and admin guards use injected auth access levels', () async {
    final router = await shoppingRouter();
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

    authenticate(router, 'dust');
    expect(staffGuard.canActivate(staffRoute), isA<ShoppingProductsRoute>());
    expect(adminGuard.canActivate(adminRoute), isA<ShoppingProductsRoute>());

    authenticate(router, 'manager');
    expect(staffGuard.canActivate(staffRoute), isNull);
    expect(adminGuard.canActivate(adminRoute), isA<ShoppingProductsRoute>());

    authenticate(router, 'admin');
    expect(staffGuard.canActivate(staffRoute), isNull);
    expect(adminGuard.canActivate(adminRoute), isNull);

    expect(shoppingRouteGuards(const ShoppingProductsRoute(), router), isEmpty);
  });

  test('generated guard resolver is typed so untyped guards cannot compile',
      () async {
    final router = await shoppingRouter();

    final adminGuards = shoppingRouteGuards(const ShoppingAdminRoute(), router);
    final staffGuards = shoppingRouteGuards(const ShoppingStaffRoute(), router);

    expect(adminGuards, isA<List<RouteGuardBase<ShoppingRoutePath>>>());
    expect(staffGuards, isA<List<RouteGuardBase<ShoppingRoutePath>>>());
    expect(adminGuards.single, isA<RouteGuard<ShoppingRoutePath>>());
    expect(staffGuards.single, isA<RouteGuard<ShoppingRoutePath>>());
  });

  test('guarded admin route denies navigation through the real router',
      () async {
    final router = await shoppingRouter();
    final delegate = GeneratedRouterDelegate<ShoppingRoutePath>(
      RouterRuntimeConfig<ShoppingRoutePath>(
        router: router,
        initialRoute: const ShoppingProductsRoute(),
        parseRoute: parseShoppingRoute,
        routeLocation: shoppingRouteLocation,
        requiresAuth: shoppingRouteRequiresAuth,
        routeBranch: shoppingRouteBranch,
        resolveGuards: (route) => shoppingRouteGuards(route, router),
        buildPage: buildShoppingRoutePage,
        restoreStack: restoreShoppingRouteStack,
      ),
    );
    await delegate.debugWaitForScheduledRefresh();

    await delegate.setNewRoutePath(const ShoppingAdminRoute());

    expect(
      delegate.stack.map((route) => route.location),
      isNot(contains('/admin')),
      reason: 'AdminGuard must deny a signed-out admin deep link',
    );
  });
}
