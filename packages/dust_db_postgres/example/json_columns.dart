import 'dart:convert';

import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Storing a document in a column.
///
/// `jsonb` is parsed, indexable, and queryable: the driver decodes it to a Dart
/// `Map` on the way out, and a `String` of JSON binds on the way in. Prefer it
/// to `json`, which stores the text verbatim and re-parses on every read.
///
/// It is for data whose shape is genuinely open — a webhook payload, per-tenant
/// settings, an audit record. A field every row has and every query filters on
/// is a column; putting it in `jsonb` gives up the type, the constraint, and
/// the planner's statistics for nothing.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/json_columns.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_events', const []);
    await db.unsafe.execute(
      '''
CREATE TABLE example_events (
  id      BIGSERIAL PRIMARY KEY,
  payload JSONB NOT NULL
)''',
      const [],
    );

    await db.execute(
      r'INSERT INTO example_events (payload) VALUES ($1), ($2)',
      <Object?>[
        jsonEncode(<String, Object?>{
          'kind': 'signup',
          'plan': 'pro',
          'seats': 5,
        }),
        jsonEncode(<String, Object?>{'kind': 'signup', 'plan': 'free'}),
      ],
    );

    final payload = await db.fetchOne<Map<String, dynamic>>(
      'SELECT payload FROM example_events ORDER BY id LIMIT 1',
      const [],
      (row) => row.read<Map<String, dynamic>>('payload'),
    );
    print('payload: ${payload.unwrapOrElse((error) => throw error)}');

    // `->>` reads a field as text, so the filter is a plain bound comparison.
    // A GIN index on payload makes this cheap at size.
    final pro = await db.fetchScalar<int>(
      r"SELECT count(*) FROM example_events WHERE payload->>'plan' = $1",
      const <Object?>['pro'],
    );
    print('pro events: ${pro.unwrapOrElse((_) => -1)}');

    // A missing field reads as NULL rather than failing, which is the whole
    // trade: no schema means no error either.
    final seats = await db.fetchAll<int?>(
      "SELECT (payload->>'seats')::int AS seats FROM example_events ORDER BY id",
      const [],
      (row) => row.readNullable<int>('seats'),
    );
    print('seats: ${seats.unwrapOrElse((_) => const <int?>[])}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_events', const []);
    await db.close();
  }
}
