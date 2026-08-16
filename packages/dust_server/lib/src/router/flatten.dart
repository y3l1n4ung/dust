import 'package:meta/meta.dart';
import 'package:shelf/shelf.dart';

import '../extraction/state.dart';
import 'middleware.dart';
import 'paths.dart';
import 'router_base.dart';

/// One route with its full path, middleware, and inherited metadata.
@internal
final class FlatRoute {
  /// Wraps a resolved route.
  const FlatRoute({
    required this.method,
    required this.path,
    required this.handler,
    required this.metadata,
    this.isMount = false,
  });

  /// The uppercase HTTP method.
  final String method;

  /// The path as mounted.
  final String path;

  /// The handler with every enclosing group's middleware applied.
  final Handler handler;

  /// Metadata from every enclosing group, outermost first.
  final List<Object> metadata;

  /// Whether the handler owns everything below [path].
  final bool isMount;
}

/// Walks [root] and returns every route with its mounted path.
///
/// [root]'s own middleware is left out, since it wraps the composed router
/// rather than each route; including it here would run it twice.
@internal
List<FlatRoute> flattenRoutes(Router root) {
  final flattened = <FlatRoute>[];
  _collect(root, '', const [], const [], flattened,
      includeOwnMiddleware: false);

  _rejectUnreachable(flattened);
  return flattened;
}

/// Refuses a route the router could never reach.
///
/// Two routes conflict when they serve the same method and compile to the same
/// pattern, which covers the exact duplicate and the subtler case of two
/// parameters with different names in the same position. Left alone the second
/// one would simply never run.
void _rejectUnreachable(List<FlatRoute> routes) {
  final seen = <String, String>{};

  for (final route in routes) {
    final pattern = compilePath(route.path, isMount: route.isMount).pattern;
    final key = '${route.method} $pattern';
    final first = seen[key];

    if (first == null) {
      seen[key] = route.path;
      continue;
    }
    if (first == route.path) {
      throw StateError('duplicate route: ${route.method} ${route.path}');
    }
    throw StateError(
      'unreachable route: ${route.method} ${route.path} is shadowed by '
      '${route.method} $first',
    );
  }
}

void _collect(
  Router group,
  String prefix,
  List<Middleware> inherited,
  List<Object> inheritedMetadata,
  List<FlatRoute> into, {
  bool includeOwnMiddleware = true,
}) {
  final path = joinPaths(prefix, group.prefix);
  final chain = [
    ...inherited,
    if (stateMiddleware(group.internals.state) case final state?) state,
    if (includeOwnMiddleware) ...resolveMiddleware(group.internals.middleware),
  ];
  final metadata = [
    ...inheritedMetadata,
    if (group.metadata case final own?) own,
  ];

  for (final route in group.internals.routes) {
    var handler = route.handler;
    for (final middleware in chain.reversed) {
      handler = middleware(handler);
    }
    into.add(
      FlatRoute(
        method: route.method,
        path: _routePath(path, route.path),
        handler: handler,
        metadata: List.unmodifiable(metadata),
        isMount: route.isMount,
      ),
    );
  }

  for (final child in group.internals.children) {
    _collect(child, path, chain, metadata, into);
  }
}

/// Joins a mount path with a route path, keeping a declared trailing slash.
///
/// `'/'` means the module's own root, so it contributes no slash. Any longer
/// path ending in `/` is asking for a trailing slash, which is a different URL
/// from the same path without one.
String _routePath(String prefix, String path) {
  final joined = joinPaths(prefix, path);
  final wantsSlash = path != '/' && path.endsWith('/');
  return wantsSlash && !joined.endsWith('/') ? '$joined/' : joined;
}
