import 'package:meta/meta.dart';
import 'package:shelf/shelf.dart';

import 'flatten.dart';
import 'compiled_route.dart';
import 'route.dart';

/// What matching a request against the route table produced.
@internal
sealed class RouteMatch {
  const RouteMatch();
}

/// A route serves this method and path.
@internal
final class Matched extends RouteMatch {
  /// Pairs the handler with what matching produced.
  const Matched(
    this.handler,
    this.parameters, {
    required this.matchedPath,
    this.mountPrefix,
  });

  /// The handler to run.
  final Handler handler;

  /// Path parameters, still percent-encoded.
  final Map<String, String> parameters;

  /// The route pattern that matched, such as `/todos/{id}`.
  final String matchedPath;

  /// The prefix to hand to the mounted handler, when the route is a mount.
  ///
  /// Set only for `mount`, where the inner handler expects paths relative to
  /// where it was mounted rather than absolute ones.
  final String? mountPrefix;
}

/// The path exists, but not for this method.
@internal
final class MethodMismatch extends RouteMatch {
  /// Carries the methods the path does serve.
  const MethodMismatch(this.allowed);

  /// Methods allowed on the path, sorted.
  final List<String> allowed;
}

/// Nothing serves this path.
@internal
final class NoMatch extends RouteMatch {
  /// The only value of its kind.
  const NoMatch();
}

/// Matches requests against a flattened route table.
///
/// A linear scan of compiled patterns, which is small enough to own. The
/// alternative meant depending on `shelf_router` and reading its private
/// context key to get path parameters back out.
@internal
final class RouteMatcher {
  /// Compiles [routes] into matchable patterns.
  RouteMatcher(List<FlatRoute> routes)
      : _routes = [
          for (var index = 0; index < routes.length; index++)
            CompiledRoute(routes[index], index),
        ] {
    for (final route in _routes) {
      _byPrefix.putIfAbsent(route.literalPrefix, () => []).add(route);
    }
  }

  final List<CompiledRoute> _routes;

  /// Routes grouped by their leading literal segments.
  final _byPrefix = <String, List<CompiledRoute>>{};

  /// The routes that could serve [path], in declaration order.
  ///
  /// A route can only match if the request walked its whole literal prefix, so
  /// the candidates are the buckets for each prefix of the request path, and
  /// nothing else. Every bucket is already in declaration order; sorting the
  /// union by declaration index keeps the one rule that decides ties.
  List<CompiledRoute> _candidates(String path) {
    final buckets = <List<CompiledRoute>>[];
    if (_byPrefix[''] case final opening?) buckets.add(opening);

    var prefix = '';
    for (final segment in path.split('/')) {
      if (segment.isEmpty) continue;
      prefix = prefix.isEmpty ? segment : '$prefix/$segment';
      if (_byPrefix[prefix] case final bucket?) buckets.add(bucket);
    }

    if (buckets.isEmpty) return const [];
    if (buckets.length == 1) return buckets.single;

    return [
      for (final bucket in buckets) ...bucket,
    ]..sort((a, b) => a.index.compareTo(b.index));
  }

  /// Finds the handler for [method] and [path].
  ///
  /// The first route in declaration order whose path matches and whose method
  /// applies wins, which is the rule `shelf_router` uses and the only one that
  /// stays predictable once mounts and any-method routes are in play. A `HEAD`
  /// request with no `HEAD` route falls back to `GET`, as clients expect.
  RouteMatch match(String method, String path) {
    final verb = method.toUpperCase();
    final allowed = <String>{};
    CompiledRoute? getRoute;
    RegExpMatch? getCaptured;

    final segmentCount = countSegments(path);

    for (final route in _candidates(path)) {
      if (!route.couldMatch(path, segmentCount)) continue;

      final captured = route.isStatic ? null : route.capture(path);
      if (!route.isStatic && captured == null) continue;

      if (route.method == verb || route.method == Route.anyMethod) {
        return route.matched(captured);
      }

      allowed.add(route.method);
      if (route.method == 'GET') {
        getRoute ??= route;
        getCaptured ??= captured;
      }
    }

    if (verb == 'HEAD' && getRoute != null) {
      return getRoute.matched(getCaptured);
    }
    if (allowed.isEmpty) return const NoMatch();

    return MethodMismatch({
      ...allowed,
      if (allowed.contains('GET')) 'HEAD',
    }.toList()
      ..sort());
  }
}
