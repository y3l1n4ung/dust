import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// What a failure looks like when nothing throws.
///
/// Every terminal returns `Result<T, SqlxError>`. A failure is a value the
/// signature already told you about, so the compiler is what reminds you to
/// handle it — not a `catch` block someone remembered to write.
///
/// [SqlxError] carries a `category` for deciding what to do, a `message` for
/// the log, and a `cause` holding the driver's own error for the detail.
/// Switching on the category is how a caller separates "retry" from "this
/// request was wrong" without matching on message text.
///
/// ```shell
/// dart run example/error_handling.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);',
    },
  );

  try {
    // A statement the database rejects: no such table.
    final broken = await db.fetchAll<int>(
      'SELECT id FROM nonexistent',
      const [],
      (row) => row.read<int>('id'),
    );
    switch (broken) {
      case Ok(:final value):
        print('unreachable: $value');
      case Err(:final error):
        print('category: ${error.category.name}');
        print('driver: ${error.driver?.name}');
    }

    // `unwrapOr` for a default, `unwrapOrElse` when the fallback needs the
    // error, `map` to keep working inside the Result.
    final count =
        await db.fetchScalar<int>('SELECT count(*) FROM users', const []);
    print('users: ${count.unwrapOr(0)}');

    // Rethrowing is a choice, not the default. `SqlxError` implements
    // `Exception`, so a boundary that really does want to throw can.
    final missing = await db.fetchOne<int>(
      r'SELECT id FROM users WHERE id = $1',
      const <Object?>[404],
      (row) => row.read<int>('id'),
    );
    try {
      missing.unwrapOrElse((error) => throw error);
    } on SqlxError catch (error) {
      print('thrown at the boundary: ${error.category.name}');
    }
  } finally {
    await db.close();
  }
}
