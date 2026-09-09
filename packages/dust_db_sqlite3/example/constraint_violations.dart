import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Telling a duplicate apart from an outage.
///
/// A UNIQUE violation arrives as a [SqlxErrorCategory.query] error — the
/// statement ran and the database refused it. The specific constraint is in
/// `cause`, the driver's own exception, because which constraints exist is the
/// schema's business and not something a portable category can enumerate.
///
/// Letting the insert fail is better than checking first: a `SELECT` before an
/// `INSERT` is a race, and the unique index has to do the work anyway. This is
/// the shape a signup handler wants — one round trip, and the duplicate
/// answered as a 409 rather than a 500.
///
/// ```shell
/// dart run example/constraint_violations.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE users (
  id    INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE
);
''',
    },
  );

  try {
    Future<String> register(String email) async {
      final result = await db.execute(
        r'INSERT INTO users (email) VALUES ($1)',
        <Object?>[email],
      );
      switch (result) {
        case Ok():
          return 'created';
        case Err(:final error):
          // Message text is the driver's, so match on what it means, not on
          // how it is worded — this check is deliberately narrow.
          final detail = '${error.cause}';
          if (error.category == SqlxErrorCategory.query &&
              detail.contains('UNIQUE constraint failed')) {
            return 'already taken';
          }
          return 'failed: ${error.category.name}';
      }
    }

    print('first: ${await register('ada@example.com')}');
    print('again: ${await register('ada@example.com')}');

    // The other way: let the database decide, and say what should happen.
    // `ON CONFLICT` moves the rule into the statement, so there is no error to
    // classify at all.
    final upserted = await db.execute(
      r'''
INSERT INTO users (email) VALUES ($1)
ON CONFLICT (email) DO NOTHING
''',
      const <Object?>['ada@example.com'],
    );
    print('upsert rows: ${upserted.unwrapOrElse((e) => throw e).rowsAffected}');
  } finally {
    await db.close();
  }
}
