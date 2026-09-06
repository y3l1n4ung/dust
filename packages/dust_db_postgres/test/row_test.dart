import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:postgres/postgres.dart' as pg;
import 'package:test/test.dart';

/// Row decoding, which is where PostgreSQL and SQLite differ most.
///
/// These need no server: a `ResultRow` is a list plus a schema, so one can be
/// built directly and the adapter exercised against it.

pg.ResultRow _row(Map<String, Object?> columns) {
  final schema = pg.ResultSchema(<pg.ResultSchemaColumn>[
    for (final name in columns.keys)
      pg.ResultSchemaColumn(
          typeOid: 0, type: pg.Type.unspecified, columnName: name),
  ]);
  return pg.ResultRow(schema: schema, values: columns.values.toList());
}

void main() {
  test('reads columns by name and by index', () {
    final row = PostgresRow(_row(<String, Object?>{'id': 7, 'name': 'Ada'}));

    expect(row.read<int>('id'), 7);
    expect(row.read<String>('name'), 'Ada');
    expect(row.readIndex<int>(0), 7);
    expect(row.readNullable<String>('name'), 'Ada');
  });

  test('a null in a required column is reported, not returned', () {
    final row = PostgresRow(_row(<String, Object?>{'name': null}));

    expect(row.readNullable<String>('name'), isNull);
    expect(() => row.read<String>('name'), throwsA(isA<SqlxError>()));
  });

  test('a missing column is reported rather than read as null', () {
    final row = PostgresRow(_row(<String, Object?>{'id': 1}));

    expect(() => row.read<String>('name'), throwsA(isA<SqlxError>()));
  });

  test('a wrong column type is reported', () {
    final row = PostgresRow(_row(<String, Object?>{'id': 'not an int'}));

    expect(() => row.read<int>('id'), throwsA(isA<SqlxError>()));
  });

  test('booleans arrive as booleans, not as 0 and 1', () {
    // The difference from SQLite: Postgres has a real boolean type, so this is
    // a cast rather than an interpretation. An integer is still accepted, since
    // a query is free to select one.
    final native = PostgresRow(_row(<String, Object?>{'live': true}));
    final numeric = PostgresRow(_row(<String, Object?>{'live': 0}));

    expect(native.readBool('live'), isTrue);
    expect(numeric.readBool('live'), isFalse);
    expect(
        PostgresRow(_row(<String, Object?>{'live': null}))
            .readBoolNullable('live'),
        isNull);
  });

  test('timestamptz arrives decoded, and is normalised to UTC', () {
    // SQLite stores ISO-8601 text that its adapter parses. Postgres decodes on
    // the wire, so this is a cast plus a timezone normalisation.
    final local = DateTime(2026, 9, 6, 12, 30);
    final row = PostgresRow(_row(<String, Object?>{'at': local}));

    final read = row.readDateTime('at');
    expect(read.isUtc, isTrue);
    expect(read, local.toUtc());
  });

  test('a text timestamp is still parsed, matching SQLite', () {
    final row = PostgresRow(
      _row(<String, Object?>{'at': '2026-09-06T12:30:00Z'}),
    );

    expect(row.readDateTime('at'), DateTime.utc(2026, 9, 6, 12, 30));
  });

  test('an unparseable timestamp is reported', () {
    final row = PostgresRow(_row(<String, Object?>{'at': 'not a date'}));

    expect(() => row.readDateTime('at'), throwsA(isA<SqlxError>()));
  });
}
