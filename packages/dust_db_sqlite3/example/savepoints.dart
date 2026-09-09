import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Undoing part of a transaction without losing the rest.
///
/// A `transaction` called on a transaction becomes a SAVEPOINT rather than a
/// second `BEGIN`, which SQLite would refuse. So a function that wants a
/// transaction can take an [Executor] and get one either way: called on a pool
/// it opens one, called inside a transaction it nests.
///
/// Rust prevents an escaped transaction handle by borrowing; Dart cannot, so
/// the handle is deactivated when the closure returns. Using it afterwards
/// fails loudly instead of writing outside the transaction.
///
/// ```shell
/// dart run example/savepoints.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': 'CREATE TABLE notes (id INTEGER PRIMARY KEY, body TEXT);',
    },
  );

  Future<Result<ExecResult, SqlxError>> write(Executor on, String body) {
    return on.execute(
      r'INSERT INTO notes (body) VALUES ($1)',
      <Object?>[body],
    );
  }

  try {
    await db.transaction<Unit>((tx) async {
      await write(tx, 'outer');

      // The inner Err releases back to the savepoint. The outer transaction is
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
      'SELECT body FROM notes ORDER BY id',
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
    await db.close();
  }
}
