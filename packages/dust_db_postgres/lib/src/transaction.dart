part of 'postgres_pool.dart';

/// Carries an `Err` out through `runTx`, which reverts only on a throw.
///
/// Dust reports a failed query as a value; `package:postgres` reverts a
/// transaction when its callback throws. Returning an `Err` from the callback
/// would commit the transaction, so it is thrown here and unwrapped outside.
final class _RollbackSignal implements Exception {
  const _RollbackSignal(this.result);

  /// The `Err` that asked for the rollback.
  final Object result;
}

/// A transaction-scoped executor.
///
/// Holds a session but no session executor: `package:postgres` gives a
/// transaction a `TxSession`, which is a `Session` and not a `SessionExecutor`,
/// so it cannot open a transaction of its own. A nested call takes the
/// savepoint path instead.
final class PostgresTransaction extends PostgresExecutor
    implements Transaction {
  /// Binds the queries to one open transaction session.
  PostgresTransaction(pg.Session session) : super(session, null);
}
