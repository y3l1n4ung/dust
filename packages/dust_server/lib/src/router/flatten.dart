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

/// One nested router's `layer` stack, and the prefix it covers.
///
/// A `layer` is defined as wrapping everything the router answers, 404s and
/// 405s included. Folding it into each route's chain instead would make it run
/// only for routes that matched — which is `routeLayer`, a different thing. So
/// the stack travels with its prefix and the composer wraps matching around it.
@internal
final class LayerScope {
  /// Pairs [prefix] with the [middleware] declared on the router mounted there.
  const LayerScope(this.prefix, this.middleware);

  /// The mounted path this stack covers, `''` for the whole application.
  final String prefix;

  /// The middleware, outermost first.
  final List<Middleware> middleware;

  /// Whether [path] falls inside this scope.
  bool covers(String path) {
    if (prefix.isEmpty) return true;
    if (!path.startsWith(prefix)) return false;

    // `/apiary` is not inside `/api`. Only a segment boundary counts.
    final rest = path.substring(prefix.length);
    return rest.isEmpty || rest.startsWith('/');
  }
}

/// Walks [root] and returns every route with its mounted path.
///
/// [root]'s own middleware is left out, since it wraps the composed router
/// rather than each route; including it here would run it twice.
@internal
List<FlatRoute> flattenRoutes(Router root) {
  final flattened = <FlatRoute>[];
  _collect(root, '', const [], const [], flattened, [],
      includeOwnMiddleware: false);

  _rejectUnreachable(flattened);
  return flattened;
}

/// Every nested router's `layer` stack, outermost prefix first.
///
/// [root]'s own stack is excluded for the same reason as above: the composer
/// applies it around the whole handler.
@internal
List<LayerScope> flattenLayerScopes(Router root) {
  final scopes = <LayerScope>[];
  _collect(root, '', const [], const [], [], scopes,
      includeOwnMiddleware: false);

  return scopes;
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
  List<FlatRoute> into,
  List<LayerScope> scopes, {
  bool includeOwnMiddleware = true,
}) {
  final path = joinPaths(prefix, group.prefix);

  // A nested `layer` becomes a scope rather than part of the route chain. In
  // the chain it would run only for a route that matched, which is what made
  // `NormalizePath` inside a `nest` silently do nothing: the layer exists to
  // rewrite a path so it *can* match, and it never got the chance.
  if (includeOwnMiddleware) {
    final own = resolveMiddleware(group.internals.middleware);
    if (own.isNotEmpty) {
      scopes.add(LayerScope(normalizePrefix(path), own));
    }
  }

  final chain = [
    ...inherited,
    if (stateMiddleware(group.internals.state) case final state?) state,
    // Always included, even for the root: a route layer is defined as running
    // for matched routes only, so it belongs to the route rather than to the
    // composed handler that also answers 404 and 405.
    ...resolveMiddleware(group.internals.routeMiddleware),
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
    _collect(child, path, chain, metadata, into, scopes);
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
