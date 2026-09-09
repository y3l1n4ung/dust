import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// The pieces together: connect, migrate, write, read, transact.
///
/// Every other file in this directory answers one question. This one is the
/// tour — the shape a small application has, with each part linked to the file
/// that goes into it.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/dust_db_postgres_example.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  // Synchronous and lazy, so a generated facade can hold one without its
  // constructor becoming a future. `connect.dart`, `connect_options.dart`
  final db = PostgresDriver.connect(
    url,
    // `migrations.dart`
    migrations: const {
      'example_0001_create_users.sql': '''
CREATE TABLE example_users (
  id    BIGSERIAL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  name  TEXT NOT NULL
);
''',
    },
  );

  try {
    // Applied here rather than at open, because the server is over a network.
    // On SQLite this has already happened and returns Ok, which is what lets
    // one startup path serve both.
    final migrated = await db.migrate();
    print('migrated: ${migrated.isOk}');

    // Every terminal returns `Result`: a failed query is a value to handle, not
    // an exception to catch. `error_handling.dart`
    //
    // `RETURNING` is how a generated id comes back — there is no lastInsertId.
    // `returning.dart`, `execute.dart`
    final created = await db.fetchOne<int>(
      r'INSERT INTO example_users (email, name) VALUES ($1, $2) RETURNING id',
      const <Object?>['ada@example.com', 'Ada'],
      (row) => row.read<int>('id'),
    );
    final int id;
    switch (created) {
      case Ok(:final value):
        id = value;
        print('created a user');
      case Err(:final error):
        print('insert failed: $error');
        return;
    }

    // `$1` is PostgreSQL's own; the SQLite driver rewrites it to `?`.
    // `placeholders.dart`, `fetch_one.dart`, `row_mappers.dart`
    final name = await db.fetchScalar<String>(
      r'SELECT name FROM example_users WHERE id = $1',
      <Object?>[id],
    );
    print('the user is ${name.unwrapOrElse((error) => throw error)}');

    // `Ok` commits, `Err` rolls back, and there is no third path out. The
    // closure holds one pooled connection throughout.
    // `transactions.dart`, `savepoints.dart`
    final renamed = await db.transaction<Unit>((tx) async {
      final updated = await tx.execute(
        r'UPDATE example_users SET name = $1 WHERE email = $2',
        const <Object?>['Grace', 'ada@example.com'],
      );
      return updated.map((_) => unit);
    });
    print('renamed: ${renamed.isOk}');

    // A real application reaches DAO methods generated from `@SqlxDao`, which
    // are these terminals with the SQL validated at build time against this
    // very schema, and the row mapping written for it.
    final all = await db.fetchAll<String>(
      'SELECT name FROM example_users ORDER BY id',
      const [],
      (row) => row.read<String>('name'),
    );
    print('users: ${all.unwrapOrElse((_) => const <String>[])}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.unsafe.execute(
      r'DELETE FROM __dust_schema_migrations WHERE name LIKE $1',
      const <Object?>['example_%'],
    );
    await db.close();
  }
}
