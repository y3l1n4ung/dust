import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// The escape hatch, and why it is awkward.
///
/// [Sqlite3UnsafeSql] runs SQL that build-time validation never sees: `EXPLAIN`,
/// a `PRAGMA`, a one-off administrative statement. Real cases, and rare ones.
///
/// Three things about it are deliberate. It is named to sting, so it greps. Its
/// decoder is passed explicitly, so the checked path stays the ergonomic one.
/// And a generated facade exposes it on the database, never on an [Executor] —
/// a request handler holds an executor and no cast takes it to the facade, so
/// unchecked SQL is out of a handler's reach by type rather than by convention.
///
/// Almost everything people reach for it has a checked form: an `IN` list is
/// one bound list, an optional filter is a `switch` over described queries, and
/// a sort column is a `switch` over an enum, since no dialect binds an
/// identifier.
///
/// ```shell
/// dart run example/unchecked_sql.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL);
CREATE INDEX users_email ON users (email);
''',
    },
  );
  final unsafe = Sqlite3UnsafeSql(db);

  try {
    // A query planner check — not a product query, and nothing to validate.
    final plan = await unsafe.fetchAs<String>(
      r'EXPLAIN QUERY PLAN SELECT id FROM users WHERE email = $1',
      const <Object?>['ada@example.com'],
      (row) => row.read<String>('detail'),
    );
    print('plan: ${plan.unwrapOrElse((_) => const <String>[])}');

    // Untyped rows, when even the shape is not known ahead of time.
    final rows = await unsafe.fetch('PRAGMA table_info(users)', const []);
    final columns = rows
        .unwrapOrElse((_) => const <Row>[])
        .map((row) => row.read<String>('name'))
        .toList();
    print('columns: $columns');

    final vacuumed = await unsafe.execute('VACUUM', const []);
    print('vacuumed: ${vacuumed.isOk}');
  } finally {
    await db.close();
  }
}
