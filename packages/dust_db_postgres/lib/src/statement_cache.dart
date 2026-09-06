part of 'postgres_pool.dart';

/// Prepared statements held per pooled connection, keyed by the SQL text.
///
/// `Session.execute` parses, describes, runs and closes a statement on every
/// call. Against a local server that measured 1023us for a single-row `SELECT`,
/// against 321us for the same query through a statement prepared once — the
/// describe reply is a round trip of its own, so a server answering the same
/// query per request pays for it every time.
///
/// A prepared statement belongs to the connection that parsed it, so queries go
/// through `withConnection` and each connection keeps its own. Rows of one
/// connection are never handed to another.
///
/// Bounded per connection. Dust's SQL is static literals validated at build
/// time, so the same text arrives every call and this hits; unchecked SQL can
/// be assembled at the call site, and an unbounded cache would then grow with
/// the request count rather than with the query count.
final class _StatementCache {
  /// How many statements to hold per connection before evicting.
  static const capacity = 64;

  final Map<pg.Connection, Map<String, pg.Statement>> _byConnection =
      <pg.Connection, Map<String, pg.Statement>>{};

  /// Runs [sql] on [connection], preparing and keeping a statement for it.
  ///
  /// PostgreSQL refuses a held statement whose result type changed under it —
  /// `cached plan must not change result type`, which a migration applied while
  /// the process runs will cause. That is the one case worth retrying: the
  /// statement is dropped and parsed again, so a schema change costs one failed
  /// call rather than every call after it.
  Future<pg.Result> run(
    pg.Connection connection,
    String sql,
    List<Object?> parameters,
  ) async {
    final statement = await _statementFor(connection, sql);
    try {
      return await statement.run(parameters);
    } on pg.ServerException catch (error) {
      if (!_isStalePlan(error)) rethrow;
      await _forget(connection, sql);
      final parsed = await _statementFor(connection, sql);
      return parsed.run(parameters);
    }
  }

  /// Releases every held statement.
  ///
  /// Closing the pool closes its connections, which drops their statements with
  /// them. Doing it here keeps the cache from handing out a statement belonging
  /// to a connection that is already gone.
  Future<void> close() async {
    final statements = <pg.Statement>[
      for (final connection in _byConnection.values) ...connection.values,
    ];
    _byConnection.clear();
    for (final statement in statements) {
      // A connection that is already gone took its statements with it, and
      // saying so here would hide the reason the pool was closed.
      try {
        await statement.dispose();
      } catch (_) {}
    }
  }

  Future<pg.Statement> _statementFor(
    pg.Connection connection,
    String sql,
  ) async {
    final held = _byConnection.putIfAbsent(
      connection,
      () => <String, pg.Statement>{},
    );

    final cached = held.remove(sql);
    if (cached != null) {
      // Reinserting makes this the most recently used.
      held[sql] = cached;
      return cached;
    }

    final prepared = await connection.prepare(pg.Sql(sql));
    held[sql] = prepared;
    if (held.length > capacity) {
      final evicted = held.remove(held.keys.first);
      if (evicted != null) await evicted.dispose();
    }
    return prepared;
  }

  Future<void> _forget(pg.Connection connection, String sql) async {
    final stale = _byConnection[connection]?.remove(sql);
    if (stale == null) return;
    try {
      await stale.dispose();
    } catch (_) {}
  }

  /// Whether [error] is PostgreSQL refusing a statement the schema outgrew.
  static bool _isStalePlan(pg.ServerException error) {
    // 0A000 is feature_not_supported, which is what the server answers a
    // prepared statement whose result type changed under it.
    return error.code == '0A000' &&
        error.message.contains('cached plan must not change result type');
  }
}
