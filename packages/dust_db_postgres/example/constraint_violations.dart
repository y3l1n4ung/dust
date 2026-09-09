import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:postgres/postgres.dart' as pg;

import 'support.dart';

/// Telling a duplicate apart from an outage.
///
/// A UNIQUE violation arrives as a [SqlxErrorCategory.query] error — the
/// statement ran and the server refused it. The specific constraint is in
/// `cause`, the driver's own exception, because which constraints exist is the
/// schema's business and not something a portable category can enumerate.
///
/// PostgreSQL gives it a SQLSTATE, and matching on that code is exact: `23505`
/// is unique_violation in every version and every locale, unlike the message,
/// which is translated. That is the advantage over SQLite, where the same check
/// has to look at the text.
///
/// Letting the insert fail is better than checking first: a `SELECT` before an
/// `INSERT` is a race, and the unique index has to do the work anyway.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/constraint_violations.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_users (id BIGSERIAL PRIMARY KEY, email TEXT NOT NULL UNIQUE)',
      const [],
    );

    Future<String> register(String email) async {
      final result = await db.execute(
        r'INSERT INTO example_users (email) VALUES ($1)',
        <Object?>[email],
      );
      switch (result) {
        case Ok():
          return 'created';
        case Err(:final error):
          final cause = error.cause;
          if (cause is pg.ServerException && cause.code == '23505') {
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
INSERT INTO example_users (email) VALUES ($1)
ON CONFLICT (email) DO NOTHING
''',
      const <Object?>['ada@example.com'],
    );
    print('upsert rows: ${upserted.unwrapOrElse((e) => throw e).rowsAffected}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.close();
  }
}
