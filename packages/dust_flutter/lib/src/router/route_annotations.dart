import 'package:flutter/material.dart' hide Route;

/// Marks the single router entrypoint for a Flutter application.
///
/// [initial] and [notFound] are route paths, not page classes. Dust validates
/// that both paths match generated `@Route` pages.
///
/// Example:
///
/// ```dart
/// @Router(initial: '/', notFound: '/404')
/// final class AppRouter extends $AppRouter {
///   AppRouter({required this.auth});
///
///   final AuthViewModel auth;
/// }
/// ```
class Router {
  /// Creates a router root annotation.
  const Router({required this.initial, required this.notFound});

  /// Initial route path, for example `/`.
  final String initial;

  /// Not-found route path, for example `/404`.
  final String notFound;
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
  final String? name;

  /// Optional layout widget type that wraps this page.
  final Type? shell;

  /// Route-specific guard types evaluated after router-level redirects.
  final List<Type> guards;

  /// Optional Flutter page transition builder for this route.
  final PageTransitionsBuilder? transition;

  /// Whether the generated page should behave like a fullscreen dialog.
  final bool fullscreenDialog;

  /// Whether Flutter should keep the page state alive when inactive.
  final bool maintainState;
}

/// Generated route tree metadata.
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
      if (route.name == name) return route;
    }
    return null;
  }

  /// Finds the first route whose page type is [page] in this subtree.
  GeneratedRoute? findByPage(Type page) {
    for (final route in depthFirst) {
      if (route.page == page) return route;
    }
    return null;
  }
}
