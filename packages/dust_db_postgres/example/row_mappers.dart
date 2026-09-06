import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Turning a row into a Dart object.
///
/// A row terminal takes a `RowMapper<T>` — one function from [Row] to your
/// type. `@Derive([FromRow()])` writes it for you and names it `$UserFromRow`,
/// which is what lets the analyzer reject a row type that has no mapping. The
/// hand-written form below is what that generates, and what you write for a
/// type Dust does not own.
///
/// PostgreSQL decodes on the wire, so a `boolean` arrives as a `bool` and a
/// `timestamptz` as a `DateTime`. The mapper casts where the SQLite one has to
/// interpret.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/row_mappers.dart
/// ```
class User {
  const User({
    required this.id,
    required this.email,
    required this.active,
    required this.createdAt,
    this.displayName,
  });

  /// Reads one `example_users` row. `read` throws on NULL and `readNullable`
  /// admits it, so the column's nullability is stated here rather than
  /// discovered in production.
  factory User.fromRow(Row row) {
    return User(
      id: row.read<int>('id'),
      email: row.read<String>('email'),
      active: row.readBool('active'),
      createdAt: row.readDateTime('created_at'),
      displayName: row.readNullable<String>('display_name'),
    );
  }

  final int id;
  final String email;
  final bool active;
  final DateTime createdAt;
  final String? displayName;

  @override
  String toString() =>
      'User($email, active: $active, name: ${displayName ?? '-'})';
}

Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.unsafe.execute(
      '''
CREATE TABLE example_users (
  id           BIGSERIAL PRIMARY KEY,
  email        TEXT NOT NULL,
  display_name TEXT,
  active       BOOLEAN NOT NULL DEFAULT TRUE,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
)''',
      const [],
    );
    await db.execute(
      r'INSERT INTO example_users (email, display_name) VALUES ($1, $2), ($3, $4)',
      const <Object?>['ada@example.com', 'Ada', 'anon@example.com', null],
    );

    final users = await db.fetchAll<User>(
      'SELECT * FROM example_users ORDER BY id',
      const [],
      User.fromRow,
    );
    for (final user in users.unwrapOrElse((_) => const <User>[])) {
      print(user);
    }
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.close();
  }
}
