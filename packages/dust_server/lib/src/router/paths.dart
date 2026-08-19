import 'path_segment.dart';

/// Normalizes one path segment into `''` or `/segment` form.
///
/// Empty input, `'/'`, and `'//'` all collapse to `''` so that joining is
/// associative and a controller mounted at the root does not gain a trailing
/// slash.
String normalizePrefix(String value) {
  var normalized = value.trim();
  if (normalized.isEmpty) return '';
  if (!normalized.startsWith('/')) normalized = '/$normalized';
  while (normalized.length > 1 && normalized.endsWith('/')) {
    normalized = normalized.substring(0, normalized.length - 1);
  }
  normalized = normalized.replaceAll(RegExp('/+'), '/');
  return normalized == '/' ? '' : normalized;
}

/// Joins two path segments, normalizing both.
String joinPaths(String prefix, String suffix) {
  final joined = '${normalizePrefix(prefix)}${normalizePrefix(suffix)}';
  return joined.isEmpty ? '/' : joined;
}

/// The parameter names a route path declares, in order.
List<String> parameterNames(String path) {
  return [
    for (final segment in _segments(path))
      if (segment
          case ParameterSegment(:final name) ||
              ConstrainedSegment(:final name) ||
              CatchAllSegment(:final name))
        name,
  ];
}

/// Compiles a route path into a matcher.
///
/// A `{name}` segment matches one segment, `{name|pattern}` matches one segment
/// against its own pattern, and `{*name}` swallows the rest of the path. When
/// [isMount] is set the path is a prefix and everything below it matches, which
/// is what hands a subtree to another handler.
RegExp compilePath(String path, {bool isMount = false}) {
  final pattern = StringBuffer('^');
  var afterCatchAll = false;

  for (final segment in _segments(path)) {
    if (afterCatchAll) {
      throw ArgumentError('a catch-all must be the last segment of "$path"');
    }

    pattern.write('/');
    pattern.write(switch (segment) {
      LiteralSegment(:final text) => RegExp.escape(text),
      ParameterSegment() => '([^/]+)',
      ConstrainedSegment(:final pattern) => '($pattern)',
      CatchAllSegment() => '(.*)',
    });
    afterCatchAll = segment is CatchAllSegment;
  }

  pattern.write(switch (_tailOf(path, pattern.length == 1, isMount)) {
    // A mount owns everything below its prefix, and at the root that is every
    // path. `/?` alone matched only the bare root, which made `mount('/', ...)`
    // — the way a single-page build is served — answer 404 for every deep link
    // and every asset.
    _PathTail.rootMount => '/?.*',
    _PathTail.root => '/',
    _PathTail.subtree => '(?:/.*)?',
    _PathTail.trailingSlash => '/',
    _PathTail.exact => '',
  });

  pattern.write(r'$');
  return RegExp(pattern.toString());
}

Iterable<PathSegment> _segments(String path) sync* {
  for (final raw in path.split('/')) {
    if (raw.isEmpty) continue;
    yield PathSegment.parse(raw);
  }
}

/// How a compiled path ends.
enum _PathTail {
  /// A mount at the root, which serves every path including the bare root.
  rootMount,

  /// The root path itself.
  root,

  /// A mount, which serves everything below its prefix.
  subtree,

  /// A declared trailing slash, kept so `/a/` and `/a` stay distinct.
  trailingSlash,

  /// Anything else: the path matches exactly.
  exact,
}

_PathTail _tailOf(String path, bool isRoot, bool isMount) {
  return switch ((isRoot, isMount, path.endsWith('/'))) {
    (true, true, _) => _PathTail.rootMount,
    (true, false, _) => _PathTail.root,
    (false, true, _) => _PathTail.subtree,
    (false, false, true) => _PathTail.trailingSlash,
    (false, false, false) => _PathTail.exact,
  };
}
