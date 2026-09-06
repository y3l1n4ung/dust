import 'dart:io';

import 'package:dust_db_postgres/dust_db_postgres.dart';

/// An `IN` list without dynamic SQL.
///
/// A Dart `List` binds as a PostgreSQL array, so `= ANY($1)` is one placeholder
/// over constant SQL — which means `dust db build` can validate it, unlike an
/// `IN (?, ?, ?)` assembled at the call site.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/set_membership.dart
/// ```
Future<void> main() async {
  final url = Platform.environment['DUST_DATABASE_URL'];
  if (url == null) {
    print('Set DUST_DATABASE_URL to a PostgreSQL database this may write to.');
    return;
  }

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_items', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_items (id BIGSERIAL PRIMARY KEY, name TEXT NOT NULL)',
      const [],
    );
    await db.unsafe.execute(
      r'INSERT INTO example_items (name) VALUES ($1), ($2), ($3)',
      const <Object?>['first', 'second', 'third'],
    );

    final some = await db.fetchAll<String>(
      r'SELECT name FROM example_items WHERE id = ANY($1) ORDER BY id',
      <Object?>[
        <int>[1, 3],
      ],
      (row) => row.read<String>('name'),
    );
    print('selected: ${some.unwrapOrElse((_) => const <String>[])}');

    // An empty list selects nothing rather than failing, which `IN ()` cannot
    // even express.
    final none = await db.fetchAll<String>(
      r'SELECT name FROM example_items WHERE id = ANY($1)',
      <Object?>[const <int>[]],
      (row) => row.read<String>('name'),
    );
    print('empty: ${none.unwrapOrElse((_) => const <String>[])}');

    await db.unsafe.execute('DROP TABLE IF EXISTS example_items', const []);
  } finally {
    await db.close();
  }
}
