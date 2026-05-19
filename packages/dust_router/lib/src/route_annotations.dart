import 'package:flutter/material.dart' hide Route;

/// Marks the single router entrypoint for a Flutter application.
///
/// The annotated class owns app-specific dependencies, redirects, guards, and
/// refresh behavior. Dust generates the base class referenced by [generatedBase]
/// in `route.g.dart` and exposes a `RouterConfig` for `MaterialApp.router`.
///
/// Example:
///
/// ```dart
/// @Router(initial: DashboardPage, notFound: NotFoundPage)
/// final class AppRouter extends $AppRouter {
///   AppRouter({required this.session});
///
///   final AppSession session;
/// }
/// ```
class Router {
  /// Creates a router root annotation.
  const Router({
    required this.initial,
    this.notFound,
    this.refreshListenable,
    this.generatedBase,
  });

  /// Page class used when the app starts without an explicit deep link.
  final Type initial;

  /// Optional page class used when no route matches a URL.
  final Type? notFound;

  /// Optional router field or getter name used as the generated refresh source.
  ///
  /// The value must reference a `Listenable` member on the annotated router
  /// class. When set, the generated router reevaluates redirects and guards
  /// after that listenable changes.
  final String? refreshListenable;

  /// Optional generated base class name.
  ///
  /// By default Dust derives the base name from the annotated class, for
  /// example `AppRouter` becomes `$AppRouter`.
  final String? generatedBase;
}

/// Marks a widget class as a typed route.
///
/// Dust reads the annotated widget constructor to generate a route data class.
/// Path parameters such as `:projectId` must match required constructor
/// parameters. Remaining supported constructor parameters become query params.
/// Route parameters are URL primitives only; complex objects should be loaded
/// from app state or repositories after navigation.
///
/// Example:
///
/// ```dart
/// @Route('/projects/:projectId', name: 'project', shell: AppShell)
/// final class ProjectPage extends StatelessWidget {
///   const ProjectPage({
///     super.key,
///     required this.projectId,
///     this.tab,
///   });
///
///   final int projectId;
///   final String? tab;
/// }
/// ```
class Route {
  /// Creates a page route annotation.
  const Route(
    this.path, {
    this.name,
    this.shell,
    this.guards = const [],
    this.transition,
    this.fullscreenDialog = false,
    this.maintainState = true,
  });

  /// Absolute URL path pattern, for example `/projects/:projectId`.
  final String path;

  /// Stable route name used for generated route and navigation helper names.
  ///
  /// If omitted, Dust derives the name from the page class by removing the
  /// `Page`, `Screen`, or `View` suffix and lower-camel-casing the result.
  final String? name;

  /// Optional layout widget type that wraps this page.
  final Type? shell;

  /// Route-specific guard types evaluated after router-level redirects.
  final List<Type> guards;

  /// Optional Flutter page transition builder for this route.
  ///
  /// When omitted, the app-level [PageTransitionsTheme] is used.
  final PageTransitionsBuilder? transition;

  /// Whether the generated page should behave like a fullscreen dialog.
  final bool fullscreenDialog;

  /// Whether Flutter should keep the page state alive when inactive.
  final bool maintainState;
}

/// Generated route tree metadata.
///
/// Users should not write [GeneratedRoute] by hand. Dust emits it in
/// `route.g.dart` so tests and debugging tools can inspect the generated tree.
class GeneratedRoute {
  /// Creates generated route metadata.
  const GeneratedRoute(
    this.path, {
    this.page,
    this.name,
    this.routes = const [],
    this.shell,
    this.guards = const [],
    this.transition,
    this.fullscreenDialog = false,
    this.maintainState = true,
  });

  /// Route path segment or absolute route path.
  final String path;

  /// Page class built by this generated route entry.
  final Type? page;

  /// Stable route name.
  final String? name;

  /// Nested generated routes grouped from child paths.
  final List<GeneratedRoute> routes;

  /// Optional shell widget inherited by this generated route subtree.
  final Type? shell;

  /// Guard classes for this generated route.
  final List<Type> guards;

  /// Optional Flutter page transition builder for this route.
  final PageTransitionsBuilder? transition;

  /// Whether the generated page is presented as a fullscreen dialog.
  final bool fullscreenDialog;

  /// Whether Flutter should keep the page state alive when inactive.
  final bool maintainState;

  /// Returns this route and every descendant in depth-first order.
  Iterable<GeneratedRoute> get depthFirst sync* {
    yield this;
    for (final route in routes) {
      yield* route.depthFirst;
    }
  }

  /// Finds the first route with [name] in this subtree.
  GeneratedRoute? findByName(String name) {
    for (final route in depthFirst) {
      if (route.name == name) {
        return route;
      }
    }
    return null;
  }

  /// Finds the first route whose page type is [page] in this subtree.
  GeneratedRoute? findByPage(Type page) {
    for (final route in depthFirst) {
      if (route.page == page) {
        return route;
      }
    }
    return null;
  }
}
