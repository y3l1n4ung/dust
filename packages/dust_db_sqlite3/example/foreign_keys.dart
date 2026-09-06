import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Making SQLite enforce the references you declared.
///
/// SQLite parses `REFERENCES` and then, by default, ignores it: enforcement is
/// off unless `PRAGMA foreign_keys` is on, and it is per connection rather than
/// stored in the file. So a schema can look correct for years while orphan rows
/// accumulate.
///
/// `foreignKeys: true` in [SqliteConnectOptions] turns it on at open, before
/// the first statement. PostgreSQL enforces constraints unconditionally, which
/// is why this file has no PostgreSQL counterpart.
///
/// ```shell
/// dart run example/foreign_keys.dart
/// ```
Future<void> main() async {
  const migrations = <String, String>{
    '0001.sql': '''
CREATE TABLE authors (id INTEGER PRIMARY KEY);
CREATE TABLE books (
  id        INTEGER PRIMARY KEY,
  author_id INTEGER NOT NULL REFERENCES authors(id)
);
''',
  };

  final unchecked = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: migrations,
  );
  final orphan = await unchecked.execute(
    r'INSERT INTO books (author_id) VALUES ($1)',
    const <Object?>[999],
  );
  // Accepted. The author does not exist.
  print('without enforcement, orphan accepted: ${orphan.isOk}');
  await unchecked.close();

  final enforced = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(foreignKeys: true),
    migrations: migrations,
  );
  try {
    final refused = await enforced.execute(
      r'INSERT INTO books (author_id) VALUES ($1)',
      const <Object?>[999],
    );
    print('with enforcement, orphan refused: ${refused.isErr}');
  } finally {
    await enforced.close();
  }
}
