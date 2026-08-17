import 'package:meta/meta.dart';
import 'package:shelf/shelf.dart';

import 'flatten.dart';
import 'matcher.dart';
import 'path_segment.dart';
import 'paths.dart';

/// One route, ready to match against.
///
/// Compilation happens once when the handler is built, so matching a request
/// is a string compare for a static path and a pattern run otherwise.
@internal
final class CompiledRoute {
  /// Compiles [route], remembering where it was declared.
  CompiledRoute(FlatRoute route, this.index)
      : method = route.method,
        handler = route.handler,
        mountPrefix = route.isMount ? route.path : null,
        matchedPath = route.path,
        _names = parameterNames(route.path),
        _staticPath = _staticPathOf(route),
        _segmentCount = _segmentCountOf(route),
        literalPrefix = _literalPrefixOf(route),
        pattern = compilePath(route.path, isMount: route.isMount);

  /// Where this route was declared, so bucketed lists can be merged in order.
  final int index;

  final String method;
  final Handler handler;
  final String? mountPrefix;

  /// The pattern this route was declared with, kept for `matchedPathOf`.
  final String matchedPath;
  final RegExp pattern;
  final List<String> _names;

  /// The leading literal segments, joined, used to bucket routes.
  ///
  /// `/api/todos/{id}` buckets under `api/todos`, so a request only ever
  /// considers routes whose literal prefix it actually walked. Empty when the
  /// path opens with a parameter, which makes the route a candidate for
  /// everything.
  final String literalPrefix;

  /// Set when the path has no parameters, so matching is a string compare.
  final String? _staticPath;

  /// Set when the path matches a fixed number of segments, so a request with
  /// the wrong count is rejected without touching the pattern.
  final int? _segmentCount;

  /// Whether this route could match [path], without running the pattern.
  ///
  /// Both checks are exact rather than heuristic: a route that passes still
  /// runs the pattern, and one that fails could not have matched.
  bool couldMatch(String path, int segmentCount) {
    if (_staticPath case final staticPath?) return staticPath == path;
    if (_segmentCount case final count?) return count == segmentCount;
    return true;
  }

  Matched matched(RegExpMatch? captured) {
    return Matched(
      handler,
      captured == null ? const {} : _parameters(captured),
      matchedPath: matchedPath,
      mountPrefix: mountPrefix,
    );
  }

  /// The match for [path], or `null` when this route does not serve it.
  RegExpMatch? capture(String path) => pattern.firstMatch(path);

  /// Whether matching needs the pattern at all.
  bool get isStatic => _staticPath != null;

  Map<String, String> _parameters(RegExpMatch captured) {
    if (_names.isEmpty) return const {};
    return {
      for (var index = 0; index < _names.length; index++)
        _names[index]: captured.group(index + 1) ?? '',
    };
  }

  static String _literalPrefixOf(FlatRoute route) {
    final literals = <String>[];
    for (final raw in route.path.split('/')) {
      if (raw.isEmpty) continue;
      if (PathSegment.parse(raw) case LiteralSegment(:final text)) {
        literals.add(text);
      } else {
        break;
      }
    }
    return literals.join('/');
  }

  static String? _staticPathOf(FlatRoute route) {
    if (route.isMount || parameterNames(route.path).isNotEmpty) return null;
    final normalized = route.path == '/' ? '/' : route.path;
    return normalized;
  }

  /// The fixed number of segments this route matches, when it has one.
  ///
  /// Only literal and plain `{name}` segments match exactly one segment each.
  /// A catch-all takes any number, and a constrained parameter can span
  /// slashes if its pattern allows, so neither has a fixed count.
  static int? _segmentCountOf(FlatRoute route) {
    if (route.isMount) return null;

    var count = 0;
    for (final raw in route.path.split('/')) {
      if (raw.isEmpty) continue;
      switch (PathSegment.parse(raw)) {
        case LiteralSegment() || ParameterSegment():
          count++;
        case ConstrainedSegment() || CatchAllSegment():
          return null;
      }
    }
    return count;
  }
}

/// The number of non-empty segments in [path].
@internal
int countSegments(String path) {
  var count = 0;
  for (final segment in path.split('/')) {
    if (segment.isNotEmpty) count++;
  }
  return count;
}
