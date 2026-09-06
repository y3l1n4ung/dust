import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

/// Committing, reverting, and nesting.
///
/// Return `Ok` to commit and `Err` to roll back. A nested call becomes a
/// savepoint on the transaction already in progress, so an inner failure can be
/// undone without losing the outer work.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/transactions.dart
/// ```
Future<void> main() async {
  final url = Platform.environment['DUST_DATABASE_URL'];
  if (url == null) {
    print('Set DUST_DATABASE_URL to a PostgreSQL database this may write to.');
    return;
  }

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_notes', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_notes (id BIGSERIAL PRIMARY KEY, body TEXT NOT NULL)',
      const [],
    );

    Future<Result<ExecResult, SqlxError>> write(Executor on, String body) {
      return on.execute(
        r'INSERT INTO example_notes (body) VALUES ($1)',
        <Object?>[body],
      );
    }

    // Ok commits.
    await db.transaction<Unit>((tx) async {
      await write(tx, 'kept');
      return const Ok<Unit, SqlxError>(unit);
    });

    // Err rolls back, and the Err is what the caller receives.
    final refused = await db.transaction<Unit>((tx) async {
      await write(tx, 'dropped');
      return Err<Unit, SqlxError>(
        SqlxError.query('changed my mind', operation: 'example'),
      );
    });
    print('refused: ${refused.isErr}');

    // A nested transaction is a savepoint. The inner failure is undone; the
    // outer transaction still commits.
    await db.transaction<Unit>((tx) async {
      await write(tx, 'outer');
      await tx.transaction<Unit>((nested) async {
        await write(nested, 'inner');
        return Err<Unit, SqlxError>(SqlxError.query('undo', operation: 'ex'));
      });
      return const Ok<Unit, SqlxError>(unit);
    });

    final notes = await db.fetchAll<String>(
      'SELECT body FROM example_notes ORDER BY id',
      const [],
      (row) => row.read<String>('body'),
    );
    // 'dropped' and 'inner' were both rolled back.
    print('kept: ${notes.unwrapOrElse((_) => const <String>[])}');

    await db.unsafe.execute('DROP TABLE IF EXISTS example_notes', const []);
  } finally {
    await db.close();
  }
}
