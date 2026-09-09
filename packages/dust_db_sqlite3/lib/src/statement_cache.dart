part of 'sqlite_pool.dart';

/// Prepared statements kept for reuse, keyed by the SQL that produced them.
///
/// Preparing is most of what a small query costs. Measured against an in-memory
/// database, a single-row `SELECT` took 5.7us re-prepared each call and 2.2us
/// from a statement held open — so a server answering the same query on every
/// request spent more time compiling SQL than reading rows.
///
/// Dust's SQL is static string literals validated at build time, so the same
/// text arrives on every call and this hits. It is bounded anyway: unchecked
/// SQL can be assembled at the call site, and an unbounded cache would then
/// grow with the request count rather than with the query count.
///
/// Least-recently-used eviction, which Dart's insertion-ordered map gives by
/// removing and reinserting on a hit.
final class _StatementCache {
  /// How many statements to hold before evicting the least recently used.
  ///
  /// Far above the number of distinct queries a package declares, and low
  /// enough that a program assembling unchecked SQL cannot grow this without
  /// bound.
  static const capacity = 128;

  final Map<String, sqlite.PreparedStatement> _entries =
      <String, sqlite.PreparedStatement>{};

  /// Returns a statement for [sql], preparing and keeping one when needed.
  ///
  /// The returned statement belongs to the cache: run it and leave it open.
  sqlite.PreparedStatement statementFor(sqlite.Database database, String sql) {
    final cached = _entries.remove(sql);
    if (cached != null) {
      // Reinserting makes this the most recently used.
      _entries[sql] = cached;
      return cached;
    }

    // `persistent` tells SQLite the statement is worth holding on to, which is
    // the whole point of caching it.
    final prepared = database.prepare(sql, persistent: true);
    _entries[sql] = prepared;
    if (_entries.length > capacity) {
      _entries.remove(_entries.keys.first)?.close();
    }
    return prepared;
  }

  /// Closes every held statement.
  ///
  /// Called before the database is closed. Closing the database would close
  /// them anyway; doing it here keeps the cache from handing out a statement
  /// belonging to a database that is already gone.
  void close() {
    for (final statement in _entries.values) {
      statement.close();
    }
    _entries.clear();
  }
}
