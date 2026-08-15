# Routing Guards

Guards protect specific routes after the router has parsed the incoming URL and
after router-level redirects have run.

## Redirects Before Guards

Use router-level `redirect` for global decisions, such as signed-out users. Use
guards for route-specific decisions, such as admin access, cart readiness, or
feature entitlement:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class ShopRouter extends $ShopRouter {
  ShopRouter({required this.auth});

  final AuthViewModel auth;

  @override
  ShopRoute? redirect(ShopRoute route) {
    if (!auth.state.isAuthenticated && route.requiresAuth) {
      return LoginRoute(redirectPath: route.location);
    }
    return null;
  }
}
```

The router redirect runs first. If it returns another route, Dust evaluates the
new route instead of continuing guards for the original route.

## Sync Guards

Use guards for route-specific access checks:

```dart
final class AdminGuard implements RouteGuard<ShopRoute> {
  const AdminGuard(this.auth);

  final AuthViewModel auth;

  @override
  ShopRoute? canActivate(ShopRoute route) {
    return auth.state.isAdmin ? null : const HomeRoute();
  }
}

@AppRoute('/admin', name: 'admin', guards: [AdminGuard])
final class AdminPage extends StatelessWidget {
  const AdminPage({super.key});
}
```

A guard returns `null` to allow navigation or another route to redirect.
Implement `AsyncRouteGuard<ShopRoute>` when the decision needs a `Future`.
Mixed sync and async guards run in annotation order, and the first redirect
wins.

## Async Guards

Use an async guard when the decision must load fresh data:

```dart
final class CheckoutReadyGuard implements AsyncRouteGuard<ShopRoute> {
  const CheckoutReadyGuard(this.cart);

  final CartViewModel cart;

  @override
  Future<ShopRoute?> canActivate(ShopRoute route) async {
    await cart.refreshIfStale();
    return cart.state.items.isEmpty ? const CartRoute() : null;
  }
}

@AppRoute('/checkout', name: 'checkout', guards: [CheckoutReadyGuard])
final class CheckoutPage extends StatelessWidget {
  const CheckoutPage({super.key});
}
```

Keep async guards short. Long-running confirmation flows belong in the page or
in a route result flow, not in navigation gating.

## Multiple Guards

List guards from cheapest to most specific. Dust stops at the first redirect:

```dart
@AppRoute(
  '/admin/orders/:orderId',
  name: 'adminOrder',
  guards: [AdminGuard, OrderAccessGuard],
)
final class AdminOrderPage extends StatelessWidget {
  const AdminOrderPage({super.key, required this.orderId});

  final String orderId;
}
```

In this example, `AdminGuard` can reject non-admin users before
`OrderAccessGuard` checks the individual order.

> [!TIP]
> Prefer one clear responsibility per guard. Small guards compose better and
> produce easier diagnostics.

## Contracts

Every class in `guards:` must implement one of the route guard contracts.
Generated guard lists are typed `List<RouteGuardBase<ShopRoute>>`, so a
class that implements neither is an analyzer error rather than a guard that
never runs.

Guard constructor dependencies are matched to router fields by type. Dust passes
`ShopRouter.auth` to `AdminGuard` above. Generation fails when a required
dependency is missing or ambiguous.

## Dependency Injection

Guard constructors can depend on router fields by type:

```dart
@AppRouter(initial: '/', notFound: '/404')
final class ShopRouter extends $ShopRouter {
  ShopRouter({
    required this.auth,
    required this.entitlements,
  });

  final AuthViewModel auth;
  final EntitlementsViewModel entitlements;
}

final class FeatureGuard implements RouteGuard<ShopRoute> {
  const FeatureGuard(this.entitlements);

  final EntitlementsViewModel entitlements;

  @override
  ShopRoute? canActivate(ShopRoute route) {
    return entitlements.state.hasPremium ? null : const UpgradeRoute();
  }
}
```

If two router fields have the same type and a guard asks for that type, Dust
cannot choose safely. Wrap one dependency in a distinct type or pass a single
coordinator object.

## Public Routes

Routes are protected by default. Mark public pages with an empty guard list:

```dart
@AppRoute('/login', name: 'login', guards: [])
final class LoginPage extends StatelessWidget {
  const LoginPage({super.key, this.redirectPath = '/'});

  final String redirectPath;
}

@AppRoute('/404', name: 'notFound', guards: [])
final class NotFoundPage extends StatelessWidget {
  const NotFoundPage({super.key, this.path = ''});

  final String path;
}
```

The configured not-found route is always public, but declaring `guards: []`
keeps the source obvious beside other public routes.

## Tests

Test guards at the route-data layer and through the router runtime. Route-data
tests keep the decision obvious:

```dart
test('admin guard rejects signed-out users', () {
  final auth = AuthViewModel.signedOut();
  final guard = AdminGuard(auth);

  expect(guard.canActivate(const AdminRoute()), const HomeRoute());
});
```

Runtime tests should cover composition: redirect first, guard order, async
guards, and the route exposed after a pop or browser back action.

## Deep Links

Guards run for browser refreshes and platform deep links too. This keeps direct
links from bypassing app navigation:

```dart
@AppRoute('/orders/:orderId', name: 'orderDetail', guards: [OrderAccessGuard])
final class OrderDetailPage extends StatelessWidget {
  const OrderDetailPage({super.key, required this.orderId});

  final String orderId;
}
```

When `/orders/ORDER-9` opens from a browser refresh, Dust restores the generated
stack, runs redirects, then runs `OrderAccessGuard` before committing the page.

## Guard Results

Return a route the user can act on. For example, redirect missing cart state to
the cart page, missing entitlement to an upgrade page, and denied admin access
to a stable home or forbidden route:

```dart
final class OrderAccessGuard implements RouteGuard<ShopRoute> {
  const OrderAccessGuard(this.orders);

  final OrdersViewModel orders;

  @override
  ShopRoute? canActivate(ShopRoute route) {
    if (route is! OrderDetailRoute) {
      return null;
    }
    return orders.canOpen(route.orderId)
        ? null
        : const NotFoundRoute(path: '/orders');
  }
}
```

Avoid redirecting back to the same route. That creates a redirect cycle and the
router reports it through `onException`.

## Forbidden Pages

Use a forbidden route when the target exists but the current user cannot open
it. Use not-found when exposing the target would leak information:

```dart
@AppRoute('/forbidden', name: 'forbidden', guards: [])
final class ForbiddenPage extends StatelessWidget {
  const ForbiddenPage({super.key});
}

final class StaffGuard implements RouteGuard<ShopRoute> {
  const StaffGuard(this.auth);

  final AuthViewModel auth;

  @override
  ShopRoute? canActivate(ShopRoute route) {
    return auth.state.isStaff ? null : const ForbiddenRoute();
  }
}
```

> [!TIP]
> Prefer a stable forbidden route for normal role checks. Prefer not-found for
> private resources where confirming the URL exists is itself sensitive.

## Diagnostics

Enable router diagnostics when debugging guard behavior:

```dart
@override
bool get debugLogDiagnostics => true;
```

Diagnostics include route names, guard decisions, redirect targets, shell and
branch metadata, and committed stacks. Keep them disabled in normal use.

> [!NOTE]
> Route guards control navigation, not backend authorization. Enforce access on
> the server as well.
