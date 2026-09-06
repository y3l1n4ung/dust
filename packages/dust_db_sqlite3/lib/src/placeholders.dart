/// SQL placeholder rewriting for SQLite.
///
/// Its own library rather than a part of the driver: this is pure text handling
/// with no database in it, and keeping it separable is what lets the cases be
/// tested directly against the ones `dust_db_plugin` checks at build time. Not
/// exported from `dust_db_sqlite3.dart` — it is internal to the driver.
library;

/// One SQL statement rewritten for SQLite, with the bind order it implies.
final class PlaceholderRewrite {
  /// Records a rewritten statement and the bind order it implies.
  const PlaceholderRewrite(this.sql, this.parameterOrder);

  /// SQL with every `$n` placeholder replaced by `?`.
  final String sql;

  /// One-based user parameter index per emitted `?`, in bind order.
  ///
  /// Empty when the statement holds no `$n` at all, which is how SQL written
  /// with native `?` placeholders passes through untouched.
  final List<int> parameterOrder;
}

/// Rewrites of SQL already seen, keyed by the source text.
///
/// Checked queries come from static string literals, so the same text arrives
/// on every call and the scan runs once per statement for the life of the
/// process. Statement preparation dominates either way; this keeps the scan
/// from being repeated work on a hot path.
final Map<String, PlaceholderRewrite> _rewriteCache =
    <String, PlaceholderRewrite>{};

/// Returns [sql] with `$n` placeholders rewritten to SQLite's `?`.
///
/// `$n` is what a query is written with on either dialect: Postgres takes it
/// unchanged, and SQLite gets this. Doing it here rather than in generated code
/// is what keeps one behaviour for a `@SqlxDao` method and an inline query, and
/// what lets the same text be validated once and run on either driver.
///
/// A `$n` only means a parameter where the database would bind one. Everything
/// that can hold those characters without meaning that is copied through byte
/// for byte: string literals, quoted identifiers, both comment forms, and
/// Postgres dollar-quoted bodies. This mirrors `rewrite_sqlite_placeholders` in
/// `dust_db_plugin`, which checks the same text at build time.
PlaceholderRewrite rewritePlaceholders(String sql) {
  final cached = _rewriteCache[sql];
  if (cached != null) return cached;

  final buffer = StringBuffer();
  final order = <int>[];
  var index = 0;
  while (index < sql.length) {
    final verbatim = _verbatimSpanEnd(sql, index);
    if (verbatim != null) {
      buffer.write(sql.substring(index, verbatim));
      index = verbatim;
      continue;
    }
    final placeholder = _placeholderAt(sql, index);
    if (placeholder != null) {
      order.add(placeholder.index);
      buffer.write('?');
      index = placeholder.end;
      continue;
    }
    buffer.write(sql[index]);
    index += 1;
  }

  final rewrite = PlaceholderRewrite(
    order.isEmpty ? sql : buffer.toString(),
    order,
  );
  _rewriteCache[sql] = rewrite;
  return rewrite;
}

/// Reorders [parameters] into the bind order [rewrite] asks for.
///
/// A `$1` used twice binds its argument twice, and `$2 ... $1` binds them in
/// the order the statement reads them. Both fall out of the placeholder order
/// rather than needing the caller to know either.
List<Object?> orderParameters(
  PlaceholderRewrite rewrite,
  List<Object?> parameters,
) {
  if (rewrite.parameterOrder.isEmpty) return parameters;
  return <Object?>[
    for (final index in rewrite.parameterOrder)
      if (index <= parameters.length)
        parameters[index - 1]
      else
        throw PlaceholderBindError(
          'SQL binds \$$index but only ${parameters.length} '
          'argument${parameters.length == 1 ? ' was' : 's were'} supplied.',
          rewrite.sql,
        ),
  ];
}

/// A `$n` placeholder and the offset just past it.
final class _Placeholder {
  const _Placeholder(this.index, this.end);

  final int index;
  final int end;
}

/// Reads a `$n` placeholder at [start], or null when there is none.
_Placeholder? _placeholderAt(String sql, int start) {
  if (sql.codeUnitAt(start) != _dollar) return null;
  var end = start + 1;
  while (end < sql.length && _isDigit(sql.codeUnitAt(end))) {
    end += 1;
  }
  if (end == start + 1) return null;
  final index = int.parse(sql.substring(start + 1, end));
  // A `$0` is not a placeholder any dialect binds, and treating it as one would
  // read past the front of the argument list.
  if (index == 0) return null;
  return _Placeholder(index, end);
}

/// Returns the end of the construct at [start] that must be copied unchanged.
int? _verbatimSpanEnd(String sql, int start) {
  final unit = sql.codeUnitAt(start);
  if (unit == _singleQuote || unit == _doubleQuote) {
    return _quotedEnd(sql, start, unit);
  }
  if (sql.startsWith('--', start)) {
    final newline = sql.indexOf('\n', start);
    return newline == -1 ? sql.length : newline;
  }
  if (sql.startsWith('/*', start)) {
    // Block comments do not nest, matching SQLite: the first `*/` ends the
    // comment however many `/*` preceded it.
    final close = sql.indexOf('*/', start + 2);
    return close == -1 ? sql.length : close + 2;
  }
  return _dollarQuoteEnd(sql, start);
}

/// Returns the end of a quoted run, where a doubled quote escapes rather than
/// closes.
int _quotedEnd(String sql, int start, int quote) {
  var index = start + 1;
  while (index < sql.length) {
    if (sql.codeUnitAt(index) != quote) {
      index += 1;
      continue;
    }
    if (index + 1 < sql.length && sql.codeUnitAt(index + 1) == quote) {
      index += 2;
      continue;
    }
    return index + 1;
  }
  // An unterminated literal runs to the end of the statement rather than
  // erroring here, leaving the database to reject the SQL the user wrote.
  return sql.length;
}

/// Returns the end of a dollar-quoted body, `$$...$$` or `$tag$...$tag$`.
int? _dollarQuoteEnd(String sql, int start) {
  if (sql.codeUnitAt(start) != _dollar) return null;
  var tagEnd = start + 1;
  while (tagEnd < sql.length && _isTagUnit(sql.codeUnitAt(tagEnd))) {
    tagEnd += 1;
  }
  // A tag may not begin with a digit, which is exactly what keeps `$1` from
  // being read as one.
  if (tagEnd > start + 1 && _isDigit(sql.codeUnitAt(start + 1))) return null;
  if (tagEnd >= sql.length || sql.codeUnitAt(tagEnd) != _dollar) return null;

  final delimiter = sql.substring(start, tagEnd + 1);
  final bodyStart = tagEnd + 1;
  final close = sql.indexOf(delimiter, bodyStart);
  // A dollar quote the database would reject — no closing tag — is not one
  // here either. Treating it as one would swallow the rest of the statement.
  if (close == -1) return null;
  return close + delimiter.length;
}

const int _dollar = 0x24;
const int _singleQuote = 0x27;
const int _doubleQuote = 0x22;
const int _underscore = 0x5f;

bool _isDigit(int unit) => unit >= 0x30 && unit <= 0x39;

bool _isTagUnit(int unit) =>
    _isDigit(unit) ||
    unit == _underscore ||
    (unit >= 0x41 && unit <= 0x5a) ||
    (unit >= 0x61 && unit <= 0x7a);

/// Raised when a statement binds a `$n` the argument list cannot satisfy.
///
/// Checked queries are validated at build time, so this is reachable only from
/// unchecked SQL — where the driver turns it into a `SqlxError` like any other
/// failure rather than letting it escape as an exception.
final class PlaceholderBindError implements Exception {
  /// Records the mismatch between a statement and its arguments.
  const PlaceholderBindError(this.message, this.sql);

  /// What went wrong, in the terms the caller wrote.
  final String message;

  /// The statement being bound.
  final String sql;

  @override
  String toString() => message;
}
