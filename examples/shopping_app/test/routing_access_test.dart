import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/route.dart';

import 'support/routing_support.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('real app public and protected route contract is explicit', () async {
    final router = await shoppingRouter();

    final publicRoutes = <ShoppingRoute>[
      const ProductsRoute(),
      const CartRoute(),
      const LoginRoute(redirectPath: '/checkout'),
      const ProductDetailRoute(productId: 1),
      const SupportChatRoute(),
      const WishlistRoute(),
    ];
    const supportRoute = SupportChatRoute();
    expect(supportRoute, isA<ShoppingRoute<bool>>());
    for (final route in publicRoutes) {
      expect(
        shoppingRouteRequiresAuth(route),
        isFalse,
        reason: '${route.runtimeType}',
      );
    }

    final protectedRoutes = <ShoppingRoute>[
      const AdminRoute(),
      const CheckoutRoute(),
      const OrdersRoute(),
      const OrderDetailRoute(orderId: 'ORDER-1'),
      const ProfileRoute(),
      const StaffRoute(),
    ];
    for (final route in protectedRoutes) {
      expect(
        shoppingRouteRequiresAuth(route),
        isTrue,
        reason: '${route.runtimeType}',
      );
    }

    expect(shoppingRouteGuards(const CheckoutRoute(), router), isEmpty);
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
    const staffRoute = StaffRoute();
    const adminRoute = AdminRoute();

    final staffGuard =
        shoppingRouteGuards(staffRoute, router).single as StaffGuard;
    final adminGuard =
        shoppingRouteGuards(adminRoute, router).single as AdminGuard;
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

    authenticate(router, 'dust');
    expect(staffGuard.canActivate(staffRoute), isA<ProductsRoute>());
    expect(adminGuard.canActivate(adminRoute), isA<ProductsRoute>());

    authenticate(router, 'manager');
    expect(staffGuard.canActivate(staffRoute), isNull);
    expect(adminGuard.canActivate(adminRoute), isA<ProductsRoute>());

    authenticate(router, 'admin');
    expect(staffGuard.canActivate(staffRoute), isNull);
    expect(adminGuard.canActivate(adminRoute), isNull);

    expect(shoppingRouteGuards(const ProductsRoute(), router), isEmpty);
  });

  test('generated guard resolver is typed so untyped guards cannot compile',
      () async {
    final router = await shoppingRouter();

    final adminGuards = shoppingRouteGuards(const AdminRoute(), router);
    final staffGuards = shoppingRouteGuards(const StaffRoute(), router);

    expect(adminGuards, isA<List<RouteGuardBase<ShoppingRoute>>>());
    expect(staffGuards, isA<List<RouteGuardBase<ShoppingRoute>>>());
    expect(adminGuards.single, isA<RouteGuard<ShoppingRoute>>());
    expect(staffGuards.single, isA<RouteGuard<ShoppingRoute>>());
  });

  test('guarded admin route denies navigation through the real router',
      () async {
    final router = await shoppingRouter();
    final delegate = GeneratedRouterDelegate<ShoppingRoute>(
      RouterRuntimeConfig<ShoppingRoute>(
        router: router,
        initialRoute: const ProductsRoute(),
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

    await delegate.setNewRoutePath(const AdminRoute());

    expect(
      delegate.stack.map((route) => route.location),
      isNot(contains('/admin')),
      reason: 'AdminGuard must deny a signed-out admin deep link',
    );
  });
}
