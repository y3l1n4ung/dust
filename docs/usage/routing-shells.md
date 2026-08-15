# Routing Shells and Branches

Shells provide layout, and branches preserve independent navigation stacks.
They use normal Dart classes; Dust does not require a separate shell annotation.

## Shells

A shell is a widget that takes a required named `Widget child`:

```dart
final class AppShell extends StatelessWidget {
  const AppShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => Scaffold(body: child);
}

@AppRoute('/dashboard', name: 'dashboard', shell: AppShell)
final class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});
}

@AppRoute('/dashboard/orders', name: 'dashboardOrders')
final class DashboardOrdersPage extends StatelessWidget {
  const DashboardOrdersPage({super.key});
}
```

Child paths inherit the nearest parent shell, so the child above is generated as
`AppShell(child: DashboardOrdersPage())`.

> [!TIP]
> Do not create empty shell marker routes. If it does not render UI or own
> navigation state, it should not be a route.

## App Chrome

Use a shell when a group of routes shares app chrome, such as a sidebar,
bottom bar, drawer, or account banner:

```dart
final class StoreShell extends StatelessWidget {
  const StoreShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Store')),
      body: child,
      bottomNavigationBar: NavigationBar(
        selectedIndex: 0,
        destinations: const [
          NavigationDestination(icon: Icon(Icons.home), label: 'Home'),
          NavigationDestination(icon: Icon(Icons.shopping_cart), label: 'Cart'),
          NavigationDestination(icon: Icon(Icons.person), label: 'Account'),
        ],
        onDestinationSelected: (index) {
          if (index == 0) {
            context.navigator.products().go();
          } else if (index == 1) {
            context.navigator.cart().go();
          } else {
            context.navigator.account().go();
          }
        },
      ),
    );
  }
}

@AppRoute('/', name: 'products', shell: StoreShell, guards: [])
final class ProductsPage extends StatelessWidget {
  const ProductsPage({super.key});
}

@AppRoute('/cart', name: 'cart')
final class CartPage extends StatelessWidget {
  const CartPage({super.key});
}

@AppRoute('/account', name: 'account')
final class AccountPage extends StatelessWidget {
  const AccountPage({super.key});
}
```

`CartPage` and `AccountPage` inherit `StoreShell` because their paths share the
same route tree. Only annotate the first route where the layout begins.

## Shell Overrides

A deeper route can choose a different shell. This is useful when an admin or
checkout area lives inside the same URL tree but needs different chrome:

```dart
final class AdminShell extends StatelessWidget {
  const AdminShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: const Text('Admin')),
    body: child,
  );
}

@AppRoute('/admin', name: 'adminDashboard', shell: AdminShell)
final class AdminDashboardPage extends StatelessWidget {
  const AdminDashboardPage({super.key});
}

@AppRoute('/admin/orders', name: 'adminOrders')
final class AdminOrdersPage extends StatelessWidget {
  const AdminOrdersPage({super.key});
}
```

`AdminOrdersPage` inherits `AdminShell`, not `StoreShell`, because the nearest
parent route with a shell wins.

## Branches

Use `branch:` when routes need independent tab stacks. Keep `shell:` for layout
and `branch:` for navigation state:

```dart
@AppRoute('/tabs/home', name: 'tabHome', shell: AppShell, branch: 'mainTabs')
final class TabHomePage extends StatelessWidget {
  const TabHomePage({super.key});
}

@AppRoute('/tabs/home/details', name: 'tabHomeDetails')
final class TabHomeDetailsPage extends StatelessWidget {
  const TabHomeDetailsPage({super.key});
}
```

Switching to another branch and back restores the first branch's stack, so
`/tabs/home` reopens with `/tabs/home/details` still on top. Child paths inherit
the nearest parent `branch:` just as they inherit `shell:`, and Dust generates a
stable constant per branch value, such as `shopBranchMainTabs`, reused in route
metadata and debug helpers.

## Multiple Tabs

Give each tab a stable branch name when every tab owns a separate nested stack:

```dart
@AppRoute('/tabs/home', name: 'tabHome', shell: StoreShell, branch: 'homeTab')
final class HomeTabPage extends StatelessWidget {
  const HomeTabPage({super.key});
}

@AppRoute('/tabs/home/product/:id', name: 'tabHomeProduct')
final class HomeProductPage extends StatelessWidget {
  const HomeProductPage({super.key, required this.id});

  final int id;
}

@AppRoute('/tabs/orders', name: 'tabOrders', shell: StoreShell, branch: 'ordersTab')
final class OrdersTabPage extends StatelessWidget {
  const OrdersTabPage({super.key});
}

@AppRoute('/tabs/orders/:orderId', name: 'tabOrderDetail')
final class OrderDetailPage extends StatelessWidget {
  const OrderDetailPage({super.key, required this.orderId});

  final String orderId;
}
```

If the user opens `/tabs/home/product/42`, switches to orders, opens
`/tabs/orders/ORDER-9`, then returns to home, Dust restores
`/tabs/home/product/42`.

> [!IMPORTANT]
> Branch names are stable state keys. Rename them like route names: deliberately
> and with regression tests for restored stacks.

## Branch Navigation

Use `go()` for tab selection, not `push()`. A tab click should expose that
branch's current stack instead of adding another page over the active stack:

```dart
NavigationDestination(icon: Icon(Icons.receipt_long), label: 'Orders')

// In the destination handler:
context.navigator.tabOrders().go();
```

Use `push()` inside a branch for drill-in pages, pickers, or detail screens that
should complete with a result.

Without `transition`, Dust uses `MaterialPage`. With one, it creates a page route
that runs the selected `PageTransitionsBuilder` at the navigation boundary.

## Transitions Inside Shells

Transitions belong to the page route, not the shell. This keeps shared chrome in
place while the child page animates:

```dart
@AppRoute(
  '/checkout',
  name: 'checkout',
  shell: StoreShell,
  transition: BottomToTopPageTransitionsBuilder(),
)
final class CheckoutPage extends StatelessWidget {
  const CheckoutPage({super.key});
}
```

Use `fullscreenDialog: true` when the route should behave like a dialog on
platforms that expose that affordance.

## Detail Pages

Do not repeat `shell:` on every detail route. Put the shell on the list page and
let detail pages inherit it:

```dart
@AppRoute('/products', name: 'products', shell: StoreShell, guards: [])
final class ProductsPage extends StatelessWidget {
  const ProductsPage({super.key});
}

@AppRoute('/products/:id', name: 'product', guards: [])
final class ProductPage extends StatelessWidget {
  const ProductPage({super.key, required this.id});

  final int id;
}

@AppRoute('/products/:id/reviews', name: 'productReviews', guards: [])
final class ProductReviewsPage extends StatelessWidget {
  const ProductReviewsPage({super.key, required this.id});

  final int id;
}
```

Opening `/products/42/reviews` restores the list route first, then the product
route, then reviews, all inside `StoreShell`.

## Modal Sections

Use a different shell when a route family should intentionally hide normal app
chrome:

```dart
final class CheckoutShell extends StatelessWidget {
  const CheckoutShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => Scaffold(body: SafeArea(child: child));
}

@AppRoute('/checkout', name: 'checkout', shell: CheckoutShell)
final class CheckoutPage extends StatelessWidget {
  const CheckoutPage({super.key});
}

@AppRoute('/checkout/payment', name: 'checkoutPayment')
final class CheckoutPaymentPage extends StatelessWidget {
  const CheckoutPaymentPage({super.key});
}
```

That keeps checkout flow routes together without carrying the store bottom bar
through payment and confirmation screens.
