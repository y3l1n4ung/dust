# Routing Prototype

This Flutter app is the manual prototype for Dust routing generation. Treat it
as the generator contract for the Rust `dust_route_plugin` implementation.

The prototype uses Flutter Navigator 2.0 directly and does not depend on an
external routing package.

## File Layout

```text
lib/
  route.dart              # user-owned app router entrypoint
  route.g.dart            # manually written generated output prototype
  route_annotations.dart  # temporary local re-export for package annotations
  layout/app_shell.dart   # normal app shell widget
  pages/*.dart            # normal annotated app pages
```

In the real Dust implementation, users write normal app files and annotations.
Dust generates `route.g.dart`. The reusable Navigator 2.0 runtime lives in
`dust_router`.

## User-Written Router

`lib/route.dart` is the only routing entrypoint the app imports:

```dart
@Router(initial: DashboardPage, notFound: NotFoundPage)
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

## User-Written Pages

Pages stay in normal `lib/pages/*` files:

```dart
@Route('/', name: 'dashboard', shell: AppShell)
final class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});
}

@Route('/projects/:projectId', name: 'project')
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

The generator must discover these annotations workspace-wide and import pages in
`route.g.dart`. Users should not manually import every page into `route.dart`.

## Generated Output Contract

`route.g.dart` must be a standalone generated library, not a `part` file. It
owns imports and generates:

- `$AppRouter`
- `AppRoutePath`
- typed route data classes
- route metadata records
- browser URL parser
- page builder
- guard lookup
- typed navigation helpers

`dust_router` contains reusable runtime code:

- `RouterConfig`
- `RouteInformationParser`
- `RouterDelegate`
- `Navigator.pages`
- redirect and guard execution
- reactive refresh handling
- browser back and forward behavior

## Example Navigation

```dart
context.routes.project(projectId: 42, tab: 'activity').go();
context.routes.login().push();
context.routes.dashboard().replace();
```

The generated API uses one method per route and shared `go`, `push`, and
`replace` actions. This avoids generating three top-level navigation methods per
route in large apps.

Deep links covered by the prototype:

```text
/invite/abc
/projects/42?tab=activity
/billing/invoices/inv_001
/search?q=dust&page=2
```

## Validation

Run from this directory:

```bash
flutter analyze
flutter test
flutter build web
```
