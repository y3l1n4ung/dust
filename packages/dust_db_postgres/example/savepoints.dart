import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Undoing part of a transaction without losing the rest.
///
/// A `transaction` called on a transaction becomes a SAVEPOINT rather than a
/// second `BEGIN`, which PostgreSQL would warn about and ignore. So a function
/// that wants a transaction can take an [Executor] and get one either way:
/// called on a pool it opens one, called inside a transaction it nests.
///
/// It matters more here than on SQLite. A failed statement poisons a PostgreSQL
/// transaction — every later statement answers `current transaction is aborted`
/// until it unwinds — and rolling back to a savepoint is what clears that
/// without discarding the work before it.
///
/// Rust prevents an escaped transaction handle by borrowing; Dart cannot, so
/// the handle is deactivated when the closure returns. Using it afterwards
/// fails loudly instead of writing outside the transaction.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/savepoints.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);

  Future<Result<ExecResult, SqlxError>> write(Executor on, String body) {
    return on.execute(
      r'INSERT INTO example_notes (body) VALUES ($1)',
      <Object?>[body],
    );
  }

  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_notes', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_notes (id BIGSERIAL PRIMARY KEY, body TEXT NOT NULL)',
      const [],
    );

    await db.transaction<Unit>((tx) async {
      await write(tx, 'outer');

      // The inner Err rolls back to the savepoint. The outer transaction is
      // untouched and still commits below.
      await tx.transaction<Unit>((nested) async {
        await write(nested, 'inner');
        return Err<Unit, SqlxError>(
          SqlxError.query('undo just this', operation: 'example'),
        );
      });

      return const Ok<Unit, SqlxError>(unit);
    });

    final kept = await db.fetchAll<String>(
      'SELECT body FROM example_notes ORDER BY id',
      const [],
      (row) => row.read<String>('body'),
    );
    print('kept: ${kept.unwrapOrElse((_) => const <String>[])}');

    late Transaction escaped;
    await db.transaction<Unit>((tx) async {
      escaped = tx;
      return const Ok<Unit, SqlxError>(unit);
    });
    final afterwards = await write(escaped, 'too late');
    print('escaped handle refused: ${afterwards.isErr}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_notes', const []);
    await db.close();
  }
}
