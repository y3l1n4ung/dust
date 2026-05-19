# Routing Plan

## Goal

Generate a production-ready Flutter router on top of Navigator 2.0 with no
external routing package. Dust route generation should keep the user's app code
small: pages live in normal `lib/` files, annotations describe the route graph,
and generated code owns the route tree, typed route data, page imports, browser
URL parsing, guards, redirects, and navigation helpers.

The public app entrypoint is `lib/route.dart`. App code imports that file and
uses `AppRouter(...).config` with `MaterialApp.router`.

## Current State

The manually written prototype in
[`examples/routing_prototype`](../../examples/routing_prototype/README.md)
represents the generator contract:

- `@Route` annotations are placed directly on normal widget/page classes.
- `@Router` lives in `lib/route.dart`.
- `lib/route.g.dart` is a standalone generated library, not a `part` file.
- `lib/route.g.dart` imports discovered pages, shells, guards, and
  `package:dust_router/dust_router.dart`.
- `dust_router` owns the reusable Navigator 2.0 runtime behavior.
- App code imports only `route.dart`.

## Public API

Router root:

```dart
@Router(
  initial: DashboardPage,
  notFound: NotFoundPage,
  refreshListenable: 'session',
)
final class AppRouter extends $AppRouter {
  AppRouter({required this.session});

  final AppSession session;

  @override
  Listenable get refreshListenable => session;

  @override
  AppRoutePath? redirect(RouteState state) {
    if (!session.isLoggedIn && state.route.requiresAuth) {
      return LoginRoute(from: state.location);
    }
    return null;
  }
}
```

Page routes:

```dart
@Route('/', name: 'dashboard', shell: AppShell)
final class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});
}

@Route('/projects/:projectId', name: 'project', guards: [ProjectGuard])
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

App usage:

```dart
MaterialApp.router(
  routerConfig: AppRouter(session: session).config,
);
```

Generated navigation helpers are route-name-first:

```dart
context.routes.project(projectId: 42, tab: 'activity').go();
context.routes.login().push();
context.routes.dashboard().replace();
```

The generator emits one factory-like method per route name and a shared
`RouteNavigation` action object with `go`, `push`, and `replace`. It must not
emit separate `goProject`, `pushProject`, and `replaceProject` methods for every
route, because large apps can have hundreds or thousands of routes.

## Route Rules

- Every `@Route` path is absolute.
- `:param` path segments map to required constructor params with the same name.
- Remaining supported constructor params become query params.
- `Key? key` and `super.key` are ignored.
- Route params are URL primitives only: `String`, `int`, `double`, `bool`,
  nullable variants, and query-param defaults.
- Complex values are not supported as route params. Custom objects, lists, maps,
  records, functions, and `dynamic` are rejected with diagnostics.
- Route names must be unique.
- Normalized paths must be unique.
- Shells wrap the nearest generated child page subtree.
- Nested route groups are derived from path segments; users do not need to write
  synthetic parent pages.
- Guard order is router redirect first, then route-level guards.

## Generated Files

Dust emits one routing output from the router root library:

- `route.g.dart` for app-specific generated code.

`route.g.dart` generates:

- `$AppRouter`
- route data classes such as `DashboardRoute` and `ProjectRoute`
- `AppRoutePath`
- `RouteState`
- route metadata records
- stable imports for annotated pages, shells, and guard types
- browser URL parser and location builders
- page builder and transition wiring
- guard lookup
- typed `BuildContext` navigation extension with one method per route and
  shared `go`, `push`, and `replace` actions
- a private `_routePath(...)` helper backed by `Uri(pathSegments: ...)` and
  `uri.pathSegments` / `uri.queryParameters` parsers

`dust_router` provides:

- `RouterConfig`
- `RouteInformationParser`
- `RouterDelegate`
- `Navigator.pages`
- browser URL sync
- reactive `refreshListenable` handling
- redirect and guard execution
- stale async navigation cancellation
- route controller access through `BuildContext`

## Rust Implementation Plan

Detailed Rust implementation plan: [routing-rust-plugin.md](routing-rust-plugin.md).

Full feature and test checklist: [routing-feature-checklist.md](routing-feature-checklist.md).


Add Flutter router runtime package `dust_router`:

- `Router`
- `Route`
- `GeneratedRoute`
- `BottomToTopPageTransitionsBuilder`
- `PageTransitionsBuilder` transition fields using Flutter's built-in transition infrastructure

Add Rust crate `dust_route_plugin`:

- Claim `@Router` and `@Route` annotations.
- Parse annotated Dart classes workspace-wide.
- Collect route facts into workspace analysis using versioned JSON records:
  `dust_route.routes.v1` and `dust_route.routers.v1`.
- Resolve page class names, source paths, package import URIs, constructor
  params, shell types, guard types, transition builders, fullscreen dialog, and
  maintain-state metadata.
- Validate duplicate route names, duplicate normalized paths, missing route
  params, unsupported param types, missing initial page, and missing not-found
  page.
- Emit `route.g.dart` from the router root library only.
- Reuse the runtime exported by `dust_router`; do not generate a
  local `routing_core.dart`.

Suggested module shape:

```text
crates/dust_route_plugin/src/
  lib.rs
  plugin.rs
  constants.rs
  model.rs
  parse.rs
  analysis.rs
  validate.rs
  emit/
    mod.rs
    core.rs
    generated.rs
```

## Tests

Rust plugin tests:

- Parse `@Router` and `@Route` annotations.
- Collect routes from multiple Dart files.
- Generate stable imports.
- Generate route data classes, parser, metadata, and navigation helpers.
- Validate duplicate names and paths.
- Validate missing path constructor params.
- Validate unsupported route param types.
- Validate missing initial and not-found pages.
- Snapshot `route.g.dart` output and runtime API expectations.

Driver tests:

- Auxiliary routing files are emitted only from the router root.
- Cache invalidates when annotated route files change.
- Existing non-routing plugins still generate unchanged output.

Flutter prototype tests:

- `flutter analyze`
- `flutter test`
- `flutter build web`
- Deep links: `/invite/abc`, `/projects/42?tab=activity`,
  `/billing/invoices/inv_001`, `/search?q=dust&page=2`.
- Guarded admin and billing routes.
- Shell route rendering.
- Browser back and forward navigation.

## Release Criteria

- [ ] Annotation package API is documented with Dartdoc.
- [ ] Rust route plugin is registered in the Dust driver.
- [ ] Generated Dart is deterministic and analyzer-clean.
- [ ] Generated route imports are stable across platforms.
- [ ] Prototype behavior is covered by Flutter tests.
- [ ] Usage docs include route root, page annotations, guards, shells, typed
      navigation, query params, path params, deep links, and web behavior.
