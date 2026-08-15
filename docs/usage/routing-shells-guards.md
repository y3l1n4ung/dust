# Routing Shells, Branches, and Guards

Shells provide layout, branches preserve independent navigation stacks, and
guards protect specific routes. They all use normal Dart classes; Dust does not
require a separate shell annotation.

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

Without `transition`, Dust uses `MaterialPage`. With one, it creates a page route
that runs the selected `PageTransitionsBuilder` at the navigation boundary.

## Guards

Use guards for route-specific access checks:

```dart
final class AdminGuard implements RouteGuard<ShopRoutePath> {
  const AdminGuard(this.auth);

  final AuthViewModel auth;

  @override
  ShopRoutePath? canActivate(ShopRoutePath route) {
    return auth.state.isAdmin ? null : const ShopHomeRoute();
  }
}

@AppRoute('/admin', name: 'admin', guards: [AdminGuard])
final class AdminPage extends StatelessWidget {
  const AdminPage({super.key});
}
```

A guard returns `null` to allow navigation or another route to redirect.
Implement `AsyncRouteGuard<ShopRoutePath>` when the decision needs a `Future`.
Mixed sync and async guards run in annotation order, and the first redirect
wins.

Every class in `guards:` must implement one of those two contracts. Generated
guard lists are typed `List<RouteGuardBase<ShopRoutePath>>`, so a class that
implements neither is an analyzer error rather than a guard that never runs.

Guard constructor dependencies are matched to router fields by type. Dust passes
`ShopRouter.auth` to `AdminGuard` above. Generation fails when a required
dependency is missing or ambiguous.

> [!NOTE]
> Route guards control navigation, not backend authorization. Enforce access on
> the server as well.
