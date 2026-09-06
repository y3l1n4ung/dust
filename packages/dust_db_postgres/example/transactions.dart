import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Committing on success, reverting on failure.
///
/// The closure's return value decides: `Ok` commits, `Err` rolls back, and the
/// `Err` is what the caller receives. There is no `commit()` to forget and no
/// `rollback()` to reach in a `catch` — a path out of the closure that skips
/// the decision does not exist. A throw rolls back too.
///
/// The closure also holds one pooled connection for its whole body, which is
/// what makes the read below see the write above it. Two statements outside a
/// transaction have no such guarantee.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/transactions.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_accounts', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_accounts (id BIGINT PRIMARY KEY, balance BIGINT NOT NULL)',
      const [],
    );
    await db.execute(
      r'INSERT INTO example_accounts (id, balance) VALUES ($1, $2), ($3, $4)',
      const <Object?>[1, 100, 2, 0],
    );

    Future<Result<Unit, SqlxError>> transfer(int amount) {
      return db.transaction<Unit>((tx) async {
        final debit = await tx.execute(
          r'UPDATE example_accounts SET balance = balance - $1 WHERE id = 1',
          <Object?>[amount],
        );
        if (debit.isErr) return debit.map((_) => unit);

        final balance = await tx.fetchScalar<int>(
          'SELECT balance FROM example_accounts WHERE id = 1',
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
          r'UPDATE example_accounts SET balance = balance + $1 WHERE id = 2',
          <Object?>[amount],
        );
        return credit.map((_) => unit);
      });
    }

    print('small transfer: ${(await transfer(30)).isOk}');
    print('overdraft: ${(await transfer(500)).isOk}');

    final balances = await db.fetchAll<int>(
      'SELECT balance FROM example_accounts ORDER BY id',
      const [],
      (row) => row.read<int>('balance'),
    );
    print('balances: ${balances.unwrapOrElse((_) => const <int>[])}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_accounts', const []);
    await db.close();
  }
}
