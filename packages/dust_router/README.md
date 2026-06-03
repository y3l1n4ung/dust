# dust_router

Typed Flutter Navigator 2.0 router runtime and annotations for Dust.

This package contains public annotations plus the small Navigator 2.0 runtime used by generated Dust routers. App code imports this package; Dust generates only app-specific route data and builders.

Dust is not publishing this package to pub.dev right now. Use a local path
dependency while the routing generator is implemented.

```yaml
dependencies:
  dust_router:
    path: ../path/to/dust/packages/dust_router
```

## Router Entrypoint

```dart
import 'package:dust_flutter/route.dart';

@Router(
  initial: DashboardPage,
  notFound: NotFoundPage,
  refreshListenable: 'session',
)
final class AppRouter extends $AppRouter {
  AppRouter({required this.session});

  final AppSession session;
}
```

## Page Route

```dart
@Route('/projects/:projectId', name: 'project', shell: AppShell)
final class ProjectPage extends StatelessWidget {
  const ProjectPage({
    super.key,
    required this.projectId,
    this.tab,
  });

  final int projectId;
  final String? tab;
}
```

Path parameters such as `:projectId` map to required constructor parameters.
Other supported constructor parameters become query parameters.

Route parameters are URL primitives only: `String`, `int`, `double`, `bool`,
nullable variants, and constructor defaults for query parameters. Custom
objects, lists, maps, records, functions, and `dynamic` are intentionally not
supported as route params.

## Transitions

```dart
@Route(
  '/checkout/:plan',
  name: 'checkout',
  transition: BottomToTopPageTransitionsBuilder(),
  fullscreenDialog: true,
)
final class CheckoutPage extends StatelessWidget {
  const CheckoutPage({super.key, required this.plan});

  final String plan;
}
```

Dust does not define a transition registry. Use Flutter's
`PageTransitionsBuilder` directly, or omit `transition` to use the app-level
`PageTransitionsTheme`.

```dart
@Route('/admin', transition: ZoomPageTransitionsBuilder())
@Route('/login', transition: CupertinoPageTransitionsBuilder())
@Route('/dashboard')
```


## Public Runtime API

Generated routers use these runtime types from this package:

- `DustRouterBase<T>`: base class for app routers. Override `refreshListenable`, `redirect`, and `guard`.
- `DustRouterController<T>`: imperative `go`, `push`, `replace`, and `pop`.
- `DustRouteState<T>`: current URI, route, stack, initial flag, and route guards.
- `RouteGuard<T>` and `RouteGuardResult<T>`: async route guard contract.
- `RouteGuardChain<T>`: runs route guards in declaration order and stops on first block or redirect.
- `GeneratedRoute`: generated route tree metadata. Synthetic group nodes may omit `page`.

Generated code imports `package:dust_flutter/route.dart`; it should not generate a local `routing_core.dart`.

## Nested Route Metadata

Dust groups generated metadata by path segments. Long paths become a tree:

```dart
const routes = [
  GeneratedRoute('/projects', routes: [
    GeneratedRoute(':projectId', page: ProjectPage, routes: [
      GeneratedRoute('settings', page: ProjectSettingsPage),
    ]),
  ]),
];
```

Navigation and parsing remain flat and typed, while metadata stays readable for docs, tooling, and tests.
