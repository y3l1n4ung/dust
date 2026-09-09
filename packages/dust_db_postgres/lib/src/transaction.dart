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
final class _PostgresTransaction extends _PostgresSession
    implements Transaction {
  _PostgresTransaction(pg.Session session) : super(session, null);
}

/// Names savepoints uniquely within a process.
int _savepointCounter = 0;

/// Runs [fn] inside a savepoint on the transaction already in progress.
///
/// `package:postgres` exposes no savepoint API, but a `TxSession` is a
/// `Session`, so the statements are available. After a successful
/// `ROLLBACK TO SAVEPOINT` the driver clears the transaction's stale error,
/// which is what lets the enclosing transaction still commit.
///
/// Takes the session rather than living on `_PostgresSession` because only the
/// nested path needs it, and a Dart part cannot hold half a class body.
Future<Result<T, SqlxError>> _runInSavepoint<T>(
  pg.Session session,
  Future<Result<T, SqlxError>> Function(Transaction tx) fn,
) async {
  final name = 'dust_sp_${_savepointCounter++}';
  try {
    await session.execute('SAVEPOINT $name');
  } catch (error) {
    return Err<T, SqlxError>(
      _postgresTransactionError(
        'PostgreSQL savepoint failed.',
        cause: error,
        operation: 'SAVEPOINT',
      ),
    );
  }

  Result<T, SqlxError> result;
  try {
    result = await fn(_PostgresTransaction(session));
  } catch (error) {
    await _endSavepoint(session, name, rollback: true);
    return Err<T, SqlxError>(
      _postgresTransactionError(
        'PostgreSQL nested transaction failed.',
        cause: error,
        operation: 'SAVEPOINT',
      ),
    );
  }

  await _endSavepoint(session, name, rollback: result.isErr);
  return result;
}

/// Ends a savepoint, either releasing it or rolling back to it.
Future<void> _endSavepoint(
  pg.Session session,
  String name, {
  required bool rollback,
}) async {
  final command =
      rollback ? 'ROLLBACK TO SAVEPOINT $name' : 'RELEASE SAVEPOINT $name';
  try {
    await session.execute(command);
  } catch (_) {
    // The enclosing transaction owns the outcome from here: if the savepoint
    // cannot be ended the transaction is already failing, and reporting this
    // instead would hide the error that caused it.
  }
}
