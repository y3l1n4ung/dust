import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Opens an in-memory SQLite database and uses Dust's DB runtime helpers.
Future<void> main() async {
  final db = SqlitePool.open(
    ':memory:',
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
    final inserted = await queryExecute(
      'INSERT INTO users (email, name) VALUES (?, ?)',
      ['ada@example.com', 'Ada'],
    ).execute(db);
    print('inserted rows: ${inserted.rowsAffected}');

    final rows = await queryRaw(
      'SELECT id, email, name FROM users WHERE id = ?',
      [inserted.lastInsertId],
    ).fetch(db);
    final user = rows.single;
    print(
      'first user: ${user.read<String>('name')} <${user.read<String>('email')}>',
    );

    final renamed = await db.transaction((tx) async {
      await queryExecute(
        'UPDATE users SET name = ? WHERE email = ?',
        ['Grace', 'ada@example.com'],
      ).execute(tx);
      return const Ok<Unit, SqlxError>(unit);
    });

    renamed.match(
      ok: (_) => print('transaction committed'),
      err: (error) => throw StateError('transaction failed: $error'),
    );
  } finally {
    await db.close();
  }
}
