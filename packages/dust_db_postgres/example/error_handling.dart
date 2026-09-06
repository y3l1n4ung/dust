import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

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
/// Over a network there is one category more to respect than on SQLite:
/// [SqlxErrorCategory.connection] is the retryable one, and a query error is
/// not — retrying a statement the server rejected just rejects it again.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/error_handling.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    // A statement the server rejects: no such table.
    final broken = await db.fetchAll<int>(
      'SELECT id FROM example_nonexistent',
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
    final one = await db.fetchScalar<int>('SELECT 1', const []);
    print('one: ${one.unwrapOr(0)}');

    // Rethrowing is a choice, not the default. `SqlxError` implements
    // `Exception`, so a boundary that really does want to throw can.
    final missing = await db.fetchOne<int>(
      r'SELECT 1 WHERE $1',
      const <Object?>[false],
      (row) => row.readIndex<int>(0),
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
