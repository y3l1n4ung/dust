import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

/// Connects to PostgreSQL and uses Dust's DB runtime helpers.
///
/// PostgreSQL is a server, so unlike the SQLite example this one cannot bring
/// its own database. Point `DUST_DATABASE_URL` at one it may write to:
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/dust_db_postgres_example.dart
/// ```
Future<void> main() async {
  final url = Platform.environment['DUST_DATABASE_URL'];
  if (url == null) {
    print('Set DUST_DATABASE_URL to a PostgreSQL database this may write to.');
    return;
  }

  final pool = PostgresDriver.connect(
    url,
    migrations: const <String, String>{
      '0001_create_users.sql': '''
CREATE TABLE IF NOT EXISTS example_users (
  id    BIGSERIAL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  name  TEXT NOT NULL
);
''',
    },
  );

  try {
    // Migrations run here rather than while connecting: PostgreSQL is reached
    // over a network, so applying them has to be awaited.
    final migrated = await pool.migrate();
    if (migrated case Err(:final error)) {
      print('migration failed: $error');
      return;
    }

    // `$1` reaches the server as written, and every terminal returns `Result`:
    // a failed query is a value to handle, not an exception to catch.
    final inserted = await queryScalar<int>(
      r'''
INSERT INTO example_users (email, name) VALUES ($1, $2)
ON CONFLICT (email) DO UPDATE SET name = excluded.name
RETURNING id
''',
      ['ada@example.com', 'Ada'],
    ).fetchOne(pool);

    final int id;
    switch (inserted) {
      case Ok(:final value):
        id = value;
        // PostgreSQL has no `lastInsertId`; `RETURNING` is the portable answer.
        print('inserted user $id');
      case Err(:final error):
        print('insert failed: $error');
        return;
    }

    // A Dart `List` binds as a PostgreSQL array, so a set membership test is
    // one placeholder over constant SQL that Dust can validate.
    final names = await queryScalar<String>(
      r'SELECT name FROM example_users WHERE id = ANY($1)',
      [
        <int>[id],
      ],
    ).fetchOne(pool);
    print('found: ${names.unwrapOrElse((error) => 'error: $error')}');

    // Return `Ok` to commit and `Err` to roll back. A nested call becomes a
    // savepoint on the transaction already in progress.
    final renamed = await pool.transaction<Unit>((tx) async {
      final updated = await queryExecute(
        r'UPDATE example_users SET name = $1 WHERE id = $2',
        ['Grace', id],
      ).execute(tx);
      return updated.map((_) => unit);
    });
    print(renamed.isOk ? 'transaction committed' : 'transaction rolled back');
  } finally {
    await pool.close();
  }
}
