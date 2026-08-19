import 'package:meta/meta.dart';
import 'package:shelf/shelf.dart';

import '../extraction/body_reader.dart';
import '../extraction/state.dart';
import '../request/request_parts.dart';
import '../response/error_reporting.dart';
import 'flatten.dart';
import 'matcher.dart';
import 'middleware.dart';
import 'router_base.dart';
import 'unmatched.dart';

/// Builds the shelf handler for [root] and everything mounted under it.
///
/// Sealing happens first: a router that accepted a route after this ran would
/// hand back routes the composed handler never sees.
@internal
Handler composeHandler(Router root) {
  sealTree(root);

  final matcher = RouteMatcher(flattenRoutes(root));

  Future<Response> routing(Request request) async {
    final path = '/${request.url.path}';

    return switch (matcher.match(request.method, path)) {
      Matched(
        :final handler,
        :final parameters,
        :final matchedPath,
        :final mountPrefix,
      ) =>
        handler(_prepare(request, parameters, matchedPath, mountPrefix)),
      MethodMismatch(:final allowed) =>
        methodNotAllowed(request.method, path, allowed),
      NoMatch() => root.internals.fallback?.call(request) ?? notFound(path),
    };
  }

  // A nested router's `layer` wraps matching itself, not the matched route, so
  // it covers the 404s and 405s inside its prefix — and so a layer that rewrites
  // the path runs early enough for the rewritten path to match.
  var composed = _applyScopes(routing, flattenLayerScopes(root));

  final limit = root.bodyLimit;
  final chain = <Middleware>[
    if (limit != null) _bodyLimitMiddleware(limit),
    if (stateMiddleware(root.internals.state) case final state?) state,
    ...resolveMiddleware(root.internals.middleware),
  ];
  for (final middleware in chain.reversed) {
    composed = middleware(composed);
  }

  final onError = root.onError;
  if (onError == null) return composed;

  // Scoped to the request rather than set globally, so two routers in one
  // isolate do not overwrite each other's sink.
  final inner = composed;
  return (request) => ServerErrors.runWith(onError, () => inner(request));
}

/// Wraps [routing] in each scope, so a request inside a prefix runs its layers.
///
/// Applied innermost prefix first, so the outermost router's stack ends up
/// outermost — the same order `layer` gives at the top level.
///
/// A request outside a scope passes through untouched. That is what keeps a
/// layer mounted at `/admin` off the storefront.
Handler _applyScopes(Handler routing, List<LayerScope> scopes) {
  var composed = routing;

  for (final scope in scopes.reversed) {
    var wrapped = composed;
    for (final middleware in scope.middleware.reversed) {
      wrapped = middleware(wrapped);
    }

    final outside = composed;
    final inside = wrapped;
    composed = (request) => scope.covers('/${request.url.path}')
        ? inside(request)
        : outside(request);
  }

  return composed;
}

/// Closes the whole mounted subtree to further mounting.
@internal
void sealTree(Router router) {
  router.internals.sealed = true;
  for (final child in router.internals.children) {
    sealTree(child);
  }
}

Middleware _bodyLimitMiddleware(int limit) {
  return (Handler inner) {
    return (Request request) =>
        inner(request.change(context: {bodyLimitContextKey: limit}));
  };
}

/// Hands a request to a matched handler.
///
/// Path parameters ride in the context. A mounted handler also gets the mount
/// prefix moved out of the URL, so it sees the path it would see if it were
/// serving at the root.
Request _prepare(
  Request request,
  Map<String, String> parameters,
  String matchedPath,
  String? mountPrefix,
) {
  reportMatchedRoute(request, matchedPath);

  var prepared = request.change(context: {matchedPathKey: matchedPath});
  if (parameters.isNotEmpty) {
    prepared = prepared.change(context: {pathParametersKey: parameters});
  }
  if (mountPrefix != null && mountPrefix.isNotEmpty) {
    prepared = prepared.change(path: mountPrefix.substring(1));
  }
  return prepared;
}
