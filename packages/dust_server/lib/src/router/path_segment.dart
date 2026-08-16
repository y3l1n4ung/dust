/// One segment of a route path, after parsing.
///
/// Parsing once into a sealed type keeps the two consumers, pattern
/// compilation and parameter naming, matching over the same shapes instead of
/// each re-testing the raw text.
sealed class PathSegment {
  const PathSegment();

  /// Reads [raw] as whichever segment form it is written in.
  factory PathSegment.parse(String raw) {
    if (_catchAll.firstMatch(raw) case final match?) {
      return CatchAllSegment(match.group(1)!);
    }
    if (_constrained.firstMatch(raw) case final match?) {
      return ConstrainedSegment(match.group(1)!, match.group(2)!);
    }
    if (_plain.firstMatch(raw) case final match?) {
      return ParameterSegment(match.group(1)!);
    }
    return LiteralSegment(raw);
  }

  static final _plain = RegExp(r'^\{([^/}|*]+)\}$');
  static final _constrained = RegExp(r'^\{([^/}|*]+)\|(.+)\}$');
  static final _catchAll = RegExp(r'^\{\*([^/}]+)\}$');
}

/// Text that has to appear as written.
final class LiteralSegment extends PathSegment {
  /// Matches [text] exactly.
  const LiteralSegment(this.text);

  /// The literal text.
  final String text;
}

/// `{name}`: one segment, captured.
final class ParameterSegment extends PathSegment {
  /// Captures one segment as [name].
  const ParameterSegment(this.name);

  /// The parameter name.
  final String name;
}

/// `{name|pattern}`: one segment matching the caller's own pattern.
final class ConstrainedSegment extends PathSegment {
  /// Captures one segment as [name], if it matches [pattern].
  const ConstrainedSegment(this.name, this.pattern);

  /// The parameter name.
  final String name;

  /// The pattern the segment has to match, as written.
  ///
  /// It is inserted as given, so a pattern allowing `/` spans segments. That
  /// is the caller's choice, and `shelf_router` behaves the same way.
  final String pattern;
}

/// `{*name}`: the rest of the path, captured.
final class CatchAllSegment extends PathSegment {
  /// Captures everything left as [name].
  const CatchAllSegment(this.name);

  /// The parameter name.
  final String name;
}
