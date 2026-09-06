import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Turning a row into a Dart object.
///
/// A row terminal takes a `RowMapper<T>` — one function from [Row] to your
/// type. `@Derive([FromRow()])` writes it for you and names it `$UserFromRow`,
/// which is what lets the analyzer reject a row type that has no mapping. The
/// hand-written form below is what that generates, and what you write for a
/// type Dust does not own.
///
/// ```shell
/// dart run example/row_mappers.dart
/// ```
class User {
  const User({required this.id, required this.email, this.displayName});

  /// Reads one `users` row. `read` throws on NULL, `readNullable` admits it, so
  /// the column's nullability is stated here rather than discovered in
  /// production.
  factory User.fromRow(Row row) {
    return User(
      id: row.read<int>('id'),
      email: row.read<String>('email'),
      displayName: row.readNullable<String>('display_name'),
    );
  }

  final int id;
  final String email;
  final String? displayName;

  @override
  String toString() => 'User($id, $email, ${displayName ?? '-'})';
}

Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE users (
  id           INTEGER PRIMARY KEY,
  email        TEXT NOT NULL,
  display_name TEXT
);
''',
    },
  );

  try {
    await db.execute(
      r'INSERT INTO users (email, display_name) VALUES ($1, $2), ($3, $4)',
      const <Object?>['ada@example.com', 'Ada', 'anon@example.com', null],
    );

    final users = await db.fetchAll<User>(
      'SELECT id, email, display_name FROM users ORDER BY id',
      const [],
      User.fromRow,
    );
    for (final user in users.unwrapOrElse((_) => const <User>[])) {
      print(user);
    }
  } finally {
    await db.close();
  }
}
