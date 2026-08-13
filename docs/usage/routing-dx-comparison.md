# Router DX Comparison

This page records how understandable Dust routing is compared with common
Flutter routing choices. It is a developer-experience note, not a performance
benchmark.

## Human-Understandable Rate

The human-understandable rate is a 1-5 score for how quickly a Flutter
developer can read, remember, and debug the routing model.

| Score | Meaning |
| :--- | :--- |
| 5 | One obvious place to define routes, calls read like app intent, and mistakes point to the fix. |
| 4 | Mostly obvious, with one extra routing concept to learn. |
| 3 | Works well after learning the router's vocabulary. |
| 2 | Requires framework-level routing knowledge or several moving parts. |
| 1 | Hard to infer from app code without tracing internals. |

Scores are based on six DX dimensions:

- Setup cost.
- Route declaration readability.
- Navigation call readability.
- Shell and tab mental model.
- Type-safety of route arguments and results.
- Error and debugging clarity.

## Summary

| Router | Human-understandable rate | Best fit | Main DX cost |
| :--- | :--- | :--- | :--- |
| Dust router | 4.5 / 5 | Apps that want typed routes from annotated pages without another router package. | Smaller ecosystem and fewer advanced nested-router escape hatches. |
| go_router | 4.0 / 5 | Most Flutter apps that want official, URL-first routing. | Shell and stateful-shell trees add concepts like navigator keys, branches, and navigation shell containers. |
| AutoRoute | 3.8 / 5 | Apps that want mature generated typed routing with many features. | More router vocabulary, route-list structure, and generator setup to remember. |
| Beamer | 3.0 / 5 | Apps that need flexible Router/Navigator pages API composition. | More custom mental model: BeamLocations, BeamStates, Beamers, and delegates. |
| Hand-written Router/Navigator 2.0 | 2.0 / 5 | Teams needing complete low-level control. | High boilerplate and many framework concepts before app intent is visible. |

## Why Dust Scores 4.5

Dust routing is easiest to remember when the rule stays this small:

```text
@AppRouter = the app router
@AppRoute = one page route
shell: AppShell = wrap matched pages in layout
branch: 'mainTabs' = keep independent tab stack state
context.navigator.product(id: 42).push() = navigate
```

The generated call chain is close to how humans describe the app:

```text
tap product
context.navigator.product(id: 42).push()
ProductRoute(id: 42)
/products/42
ProductPage(id: 42)
```

For shell routes, Dust keeps the shell as a normal widget instead of adding a
new annotation:

```dart
final class AppShell extends StatelessWidget {
  const AppShell({required this.child, super.key});

  final Widget child;
}

@AppRoute('/dashboard', name: 'dashboard', shell: AppShell)
final class DashboardPage extends StatelessWidget {}

@AppRoute('/dashboard/orders', name: 'dashboardOrders')
final class DashboardOrdersPage extends StatelessWidget {}
```

Human call chain:

```text
dashboardOrders()
DashboardOrdersRoute()
nearest parent shell = AppShell
AppShell(child: DashboardOrdersPage())
```

For tab stacks, Dust keeps the same annotation and adds `branch:`:

```text
shell = layout
branch = preserved navigation stack
```

That is easier to remember than making app code choose between a shell route,
a stateful shell route, branch objects, navigator keys, and a branch container.

Dust loses half a point because it is still beta routing. go_router and
AutoRoute have broader ecosystem examples and more escape hatches for unusual
nested navigation.

## Compared With go_router

go_router is the baseline Flutter developers already recognize. Its core DX is
strong: routes are URL-first, `context.go('/path')` is readable, redirects are
documented, and the package is maintained by `flutter.dev`.

go_router becomes harder to explain around shells:

```text
ShellRoute = one nested Navigator around matching child routes
StatefulShellRoute = separate Navigator per branch
StatefulNavigationShell.goBranch(index: n) = switch branch
navigatorContainerBuilder = render branch Navigators
```

Dust's comparable model is smaller:

```text
shell: AppShell = layout wrapper
branch: 'tabName' = independent stack
context.navigator.tabHome().go() = switch
```

Where go_router is stronger:

- It is official, widely used, and feature-complete.
- `ShellRoute` and `StatefulShellRoute` expose lower-level Navigator control.
- Existing teams already know the URL-first API.

Where Dust is clearer:

- Route arguments become constructor parameters and generated navigation
  methods.
- Shells are plain widgets with `Widget child`.
- App code imports one `route.dart` entrypoint.

## Compared With AutoRoute

AutoRoute is the closest DX competitor because it also uses generation and
typed route objects. It has strong typed arguments, deep-link support, guards,
nested navigation, tab routers, and result handling.

AutoRoute's mental model is broader:

```text
@AutoRouterConfig
RootStackRouter
AutoRoute list
PageRouteInfo objects
AutoRouter / AutoTabsRouter
Route guards
build_runner or another builder flow
```

Dust's model is intentionally smaller:

```text
@AppRouter class
@AppRoute page annotation
generated AppRoutePath classes
context.navigator.routeName()
dust build
```

Where AutoRoute is stronger:

- Mature generated router feature set.
- More documented nested-routing and tab-routing recipes.
- More migration history and community examples.

Where Dust is clearer:

- The route lives on the page widget, not in a central route list.
- The shell API does not require a separate shell route annotation.
- Dust generation uses the same CLI as the rest of Dust features.

## Compared With Beamer

Beamer is flexible and built on the Router and Navigator pages API. It supports
arbitrary nested navigation, guards, multiple Beamers, and custom route state.

That flexibility costs readability for teams that only need normal app routes:

```text
BeamerDelegate
BeamLocation
BeamState
BeamPage
beamToNamed()
```

Dust's route files usually reveal app intent faster:

```text
@AppRoute('/orders/:id', name: 'orderDetail')
context.navigator.orderDetail(id: 'ORDER-1').push()
```

Where Beamer is stronger:

- Flexible nested Router composition.
- Useful when multiple independent Beamers are a deliberate architecture.

Where Dust is clearer:

- Less router-specific vocabulary.
- More generated type checks around page constructor arguments.
- Generated parser, formatter, guards, shell wrapping, and navigation helpers
  stay in one route output.

## Compared With Hand-Written Router/Navigator 2.0

Flutter's Router API is the foundation. It is correct for advanced apps, web
history, deep links, and multiple Navigators, but handwritten Router 2.0 makes
humans read infrastructure before they see app intent:

```text
RouteInformationParser
RouterDelegate
BackButtonDispatcher
Navigator.pages
Page objects
parse URL
restore stack
notify listeners
```

Dust generates that layer and leaves app code with:

```text
@AppRoute('/products/:id', name: 'product')
context.navigator.product(id: 42).go()
```

## DX Gaps To Keep Watching

Dust should not add more annotations unless a feature cannot be expressed with
the current two-annotation model.

Remaining DX risks:

- Advanced nested navigation may still need lower-level escape hatches.
- Branch names are strings, so docs and diagnostics must keep them easy to
  audit.
- go_router and AutoRoute have more ecosystem examples today.
- The generated file must stay readable because it is part of Dust's debugging
  story.

## Source Notes

- Flutter recommends routing packages such as go_router for advanced direct
  links or multiple Navigators, while handwritten Router gives full control:
  <https://docs.flutter.dev/ui/navigation>
- go_router documents URL-based routing, redirects, deep links, multiple
  Navigators via `ShellRoute`, and type-safe routes:
  <https://pub.dev/packages/go_router>
- go_router `ShellRoute` creates a nested Navigator for matching sub-routes:
  <https://pub.dev/documentation/go_router/latest/go_router/ShellRoute-class.html>
- go_router `StatefulShellRoute` creates separate branch Navigators for
  stateful nested navigation:
  <https://pub.dev/documentation/go_router/latest/go_router/StatefulShellRoute-class.html>
- AutoRoute documents generated typed routing, guarded routes, deep linking,
  nested navigation, and tab navigation:
  <https://pub.dev/packages/auto_route>
- Beamer describes itself as Router/Navigator pages API routing with arbitrary
  nested navigation, guards, and multiple Beamers:
  <https://pub.dev/packages/beamer>
