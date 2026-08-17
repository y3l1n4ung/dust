import 'package:meta/meta.dart';
import 'package:shelf/shelf.dart';

import '../extraction/body_reader.dart';
import '../extraction/state.dart';
import 'composer.dart';
import 'group_internals.dart';
import 'method_router.dart';
import 'middleware.dart';
import 'paths.dart';
import 'route.dart';

/// A tree of routes, composed at startup.
///
/// The shape follows axum: build one with [route], nest others under a path
/// with [nest], add middleware with [layer], and attach dependencies with
/// [withState].
///
/// ```dart
/// final app = Router()
///   ..route('/health', get(health))
///   ..nest('/api/v1', v1)
///   ..layer(Cors())
///   ..withState(repo);
/// ```
final class Router {
  /// Creates an empty router.
  ///
  /// [bodyLimit] and [onError] are read from the outermost router only, since
  /// both describe how the whole application behaves.
  Router({
    Object? metadata,
    int? bodyLimit,
    void Function(Object error, StackTrace stack)? onError,
  }) : this._(
          prefix: '',
          metadata: metadata,
          bodyLimit: bodyLimit ?? defaultBodyLimit,
          onError: onError,
        );

  /// Creates the router a generated module exposes.
  ///
  /// Both handler styles land here: a `@Controller` class exposes it from a
  /// `routes` getter, and a library of top-level handlers from a generated
  /// top-level getter.
  factory Router.module({
    String prefix = '',
    List<Route> routes = const [],
    Object? metadata,
  }) {
    return Router._(
      prefix: normalizePrefix(prefix),
      routes: routes,
      metadata: metadata,
    );
  }

  Router._({
    required this.prefix,
    this.metadata,
    List<Route> routes = const [],
    this.bodyLimit,
    this.onError,
  }) : internals = GroupInternals(routes: List.of(routes));

  /// The prefix this router contributes, normalized.
  final String prefix;

  /// Anything an add-on wants to hang off this router.
  ///
  /// `dust_server` never reads it. It is here so a package that generates API
  /// documentation, or anything else that walks the tree, can attach a
  /// description and read it back from `describe`.
  final Object? metadata;

  /// The body limit, read from the outermost router.
  final int? bodyLimit;

  /// The error sink, read from the outermost router.
  final void Function(Object error, StackTrace stack)? onError;

  /// Mutable composition state, not part of the public surface.
  @internal
  final GroupInternals internals;

  /// Serves [methods] at [path], relative to this router.
  ///
  /// ```dart
  /// final notes = Router()
  ///   ..route('/', get(listNotes).post(writeNote))
  ///   ..route('/{id}', get(readNote));
  /// ```
  void route(String path, MethodRouter methods) {
    _requireOpen();
    if (methods.handlers.isEmpty) {
      throw ArgumentError('route "$path" was given no handlers');
    }
    for (final entry in methods.handlers.entries) {
      internals.routes.add(Route(entry.key, path, entry.value));
    }
  }

  /// Hands everything under [path] to [handler].
  ///
  /// The counterpart to `shelf_router.mount` and axum's `nest_service`. The
  /// handler sees paths relative to [path], so a `shelf_static` handler, or
  /// any other framework's handler, works unchanged:
  ///
  /// ```dart
  /// app.mount('/assets', createStaticHandler('public'));
  /// ```
  void mount(String path, Handler handler) {
    _requireOpen();
    internals.routes.add(Route.mount(normalizePrefix(path), handler));
  }

  /// Mounts [router] under [path].
  void nest(String path, Router router) {
    _requireOpen();
    if (identical(router, this)) {
      throw ArgumentError('a Router cannot nest itself');
    }

    final mount = Router._(prefix: normalizePrefix(path));
    mount.internals.children.add(router);
    internals.children.add(mount);
  }

  /// Mounts [router] at this router's own path, adding no prefix.
  void merge(Router router) => nest('', router);

  /// Wraps everything below this router in [middleware].
  ///
  /// Takes a shelf [Middleware] or a const-expressible [Layer]. Layers run
  /// outermost first, in the order they were added.
  ///
  /// "Everything below" means everything this router answers, and on the
  /// **top-level** router that includes the 404 for a path nothing matched. On a
  /// router that has been [nest]ed or [merge]d it does not: the parent finds a
  /// route first, so a request that matches nothing in the subtree never reaches
  /// the subtree's layers at all.
  ///
  /// The distinction matters for anything that has to run *before* routing —
  /// `NormalizePath` above all, whose job is to rewrite a path so that it can
  /// match. Added to a nested router it silently does nothing. Put such a layer
  /// on the top-level router, above the `nest`.
  void layer(Object middleware) {
    _requireOpen();
    internals.middleware.add(middleware);
  }

  /// Wraps only the routes below this router in [middleware].
  ///
  /// The difference from [layer] is what it does **not** wrap: a request that
  /// matched no route, or matched a path but not a method, never reaches a
  /// route layer. It answers 404 or 405 without running it.
  ///
  /// That is the difference between decoration and enforcement:
  ///
  /// ```dart
  /// final app = Router()
  ///   ..layer(const RequestId())        // every answer, 404 included
  ///   ..routeLayer(RequireApiKey())     // only requests that hit a route
  ///   ..merge(routes);
  /// ```
  ///
  /// Authentication usually wants this one. A `layer` that rejects
  /// unauthenticated requests also rejects the 404 for a path that does not
  /// exist, which turns "no such route" into "not authorised" and makes a
  /// missing route look like a permissions bug.
  ///
  /// Logging and tracing usually want [layer] instead: a 404 is a request too,
  /// and one that vanishes from the log is one nobody can explain.
  void routeLayer(Object middleware) {
    _requireOpen();
    internals.routeMiddleware.add(middleware);
  }

  /// Attaches state that `@State() T` handlers below this router read.
  ///
  /// ```dart
  /// final notes = Router()
  ///   ..merge(noteRoutes)
  ///   ..withState(repo);
  /// ```
  void withState<T extends Object>(T value) {
    _requireOpen();
    internals.state[stateKeyFor<T>()] = value;
  }

  /// Answers requests no route matched, instead of the default 404.
  ///
  /// Read from the outermost router, as axum does for a top-level fallback.
  void fallback(Handler handler) {
    _requireOpen();
    internals.fallback = handler;
  }

  void _requireOpen() {
    if (internals.sealed || internals.composed != null) {
      throw StateError(
        'cannot change a router whose handler has already been built',
      );
    }
  }

  /// The composed shelf handler.
  ///
  /// Building it flattens the tree, which is where two routes claiming the same
  /// method and path are caught. Nothing earlier can catch them, since the full
  /// path only exists once everything is mounted.
  ///
  /// The result is cached, and changing the tree afterwards throws instead of
  /// producing a handler that ignores the change.
  Handler get handler => internals.composed ??= composeHandler(this);
}
