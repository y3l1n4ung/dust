import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Committing on success, reverting on failure.
///
/// The closure's return value decides: `Ok` commits, `Err` rolls back, and the
/// `Err` is what the caller receives. There is no `commit()` to forget and no
/// `rollback()` to reach in a `catch` — a path out of the closure that skips
/// the decision does not exist.
///
/// A throw rolls back too, so a bug in the closure cannot leave a transaction
/// open.
///
/// ```shell
/// dart run example/transactions.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE accounts (
  id      INTEGER PRIMARY KEY,
  balance INTEGER NOT NULL
);
''',
    },
  );

  try {
    await db.execute(
      r'INSERT INTO accounts (id, balance) VALUES ($1, $2), ($3, $4)',
      const <Object?>[1, 100, 2, 0],
    );

    Future<Result<Unit, SqlxError>> transfer(int amount) {
      return db.transaction<Unit>((tx) async {
        final debit = await tx.execute(
          r'UPDATE accounts SET balance = balance - $1 WHERE id = 1',
          <Object?>[amount],
        );
        if (debit.isErr) return debit.map((_) => unit);

        final balance = await tx.fetchScalar<int>(
          'SELECT balance FROM accounts WHERE id = 1',
          const [],
        );
        // Returning Err undoes the debit above. Both statements land or
        // neither does.
        if (balance.unwrapOr(0) < 0) {
          return Err<Unit, SqlxError>(
            SqlxError.query('insufficient funds', operation: 'transfer'),
          );
        }

        final credit = await tx.execute(
          r'UPDATE accounts SET balance = balance + $1 WHERE id = 2',
          <Object?>[amount],
        );
        return credit.map((_) => unit);
      });
    }

    print('small transfer: ${(await transfer(30)).isOk}');
    print('overdraft: ${(await transfer(500)).isOk}');

    final balances = await db.fetchAll<int>(
      'SELECT balance FROM accounts ORDER BY id',
      const [],
      (row) => row.read<int>('balance'),
    );
    print('balances: ${balances.unwrapOrElse((_) => const <int>[])}');
  } finally {
    await db.close();
  }
}
