import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Opens an in-memory SQLite database and uses Dust's DB runtime helpers.
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(foreignKeys: true),
    migrations: const {
      '0001_create_users.sql': '''
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL
);
''',
    },
  );

  try {
    // Every terminal returns `Result`: a failed query is a value to handle,
    // not an exception to catch.
    final inserted = await queryExecute(
      'INSERT INTO users (email, name) VALUES (?, ?)',
      ['ada@example.com', 'Ada'],
    ).execute(db);
    final ExecResult insertResult;
    switch (inserted) {
      case Ok(:final value):
        insertResult = value;
        print('inserted rows: ${value.rowsAffected}');
      case Err(:final error):
        print('insert failed: $error');
        return;
    }

    // A product query goes through a checked helper. `queryScalar` needs no
    // row type, so it is the smallest one a driver example can show; a real
    // application reads whole rows with `queryAs<T>` and a generated mapping.
    final name = await queryScalar<String>(
      'SELECT name FROM users WHERE id = ?',
      [insertResult.lastInsertId],
    ).fetchOne(db);
    switch (name) {
      case Ok(:final value):
        print('first user: $value');
      case Err(:final error):
        print('read failed: $error');
        return;
    }

    final renamed = await db.transaction((tx) async {
      final updated = await queryExecute(
        'UPDATE users SET name = ? WHERE email = ?',
        ['Grace', 'ada@example.com'],
      ).execute(tx);
      // Returning the `Err` rolls the transaction back.
      return updated.map((_) => unit);
    });

    renamed.match(
      ok: (_) => print('transaction committed'),
      err: (error) => throw StateError('transaction failed: $error'),
    );
  } finally {
    await db.close();
  }
}
