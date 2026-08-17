import 'package:shelf/shelf.dart';

import 'route.dart';
import 'router_base.dart';

/// The mutable half of a [Router], kept off its public surface.
///
/// Composition needs to walk children, resolve middleware, and mark routers as
/// closed to further building. None of that is anyone else's business, so it
/// lives here rather than as a dozen fields on the router itself.
final class GroupInternals {
  /// Holds the state for one router.
  GroupInternals({required this.routes});

  /// Routes declared on the router.
  final List<Route> routes;

  /// Middleware added with `layer`, outermost first.
  final middleware = <Object>[];

  /// Middleware added with `routeLayer`, which wraps matched routes only.
  final routeMiddleware = <Object>[];

  /// State attached with `withState`, keyed by type.
  final state = <String, Object>{};

  /// Routers nested inside it.
  final children = <Router>[];

  /// The fallback handler, when one was set.
  Handler? fallback;

  /// Whether a composed handler has been built over it.
  bool sealed = false;

  /// The cached handler, once built.
  Handler? composed;
}
