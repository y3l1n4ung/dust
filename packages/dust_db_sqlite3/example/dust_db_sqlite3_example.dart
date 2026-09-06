import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// The pieces together: open, migrate, write, read, transact.
///
/// Every other file in this directory answers one question. This one is the
/// tour — the shape a small application has, with each part linked to the file
/// that goes into it.
///
/// ```shell
/// dart run example/dust_db_sqlite3_example.dart
/// ```
Future<void> main() async {
  // `open_in_memory.dart`, `open_a_file.dart`, `connect_options.dart`
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(foreignKeys: true),
    // `migrations.dart`
    migrations: const {
      '0001_create_users.sql': '''
CREATE TABLE users (
  id    INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  name  TEXT NOT NULL
);
''',
    },
  );

  try {
    // Every terminal returns `Result`: a failed query is a value to handle, not
    // an exception to catch. `error_handling.dart`
    //
    // `RETURNING` reads the generated id back in the same statement, which is
    // both a round trip saved and the spelling PostgreSQL shares.
    // `returning.dart`, `execute.dart`
    final created = await db.fetchOne<int>(
      r'INSERT INTO users (email, name) VALUES ($1, $2) RETURNING id',
      const <Object?>['ada@example.com', 'Ada'],
      (row) => row.read<int>('id'),
    );
    final int id;
    switch (created) {
      case Ok(:final value):
        id = value;
        print('created user $id');
      case Err(:final error):
        print('insert failed: $error');
        return;
    }

    // `$1` on both drivers; this one rewrites it to SQLite's `?` at bind time.
    // `placeholders.dart`, `fetch_one.dart`, `row_mappers.dart`
    final name = await db.fetchScalar<String>(
      r'SELECT name FROM users WHERE id = $1',
      <Object?>[id],
    );
    print('user $id is ${name.unwrapOrElse((error) => throw error)}');

    // `Ok` commits, `Err` rolls back, and there is no third path out.
    // `transactions.dart`, `savepoints.dart`
    final renamed = await db.transaction<Unit>((tx) async {
      final updated = await tx.execute(
        r'UPDATE users SET name = $1 WHERE email = $2',
        const <Object?>['Grace', 'ada@example.com'],
      );
      return updated.map((_) => unit);
    });
    print('renamed: ${renamed.isOk}');

    // A real application reaches DAO methods generated from `@SqlxDao`, which
    // are these terminals with the SQL validated at build time and the row
    // mapping written for it.
    final all = await db.fetchAll<String>(
      'SELECT name FROM users ORDER BY id',
      const [],
      (row) => row.read<String>('name'),
    );
    print('users: ${all.unwrapOrElse((_) => const <String>[])}');
  } finally {
    await db.close();
  }
}
