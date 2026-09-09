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

  test('an index outside the row is reported', () {
    final row = PostgresRow(_row(<String, Object?>{'id': 1}));

    expect(() => row.readIndex<int>(5), throwsA(isA<SqlxError>()));
    expect(() => row.readIndexNullable<int>(-1), throwsA(isA<SqlxError>()));
  });

  test('a null at an index is reported when a value is required', () {
    final row = PostgresRow(_row(<String, Object?>{'id': null}));

    expect(row.readIndexNullable<int>(0), isNull);
    expect(() => row.readIndex<int>(0), throwsA(isA<SqlxError>()));
  });

  test('a column that is neither bool nor int is not a boolean', () {
    final row = PostgresRow(_row(<String, Object?>{'live': 'yes'}));

    expect(() => row.readBool('live'), throwsA(isA<SqlxError>()));
  });

  test('a null boolean is reported when a value is required', () {
    final row = PostgresRow(_row(<String, Object?>{'live': null}));

    expect(() => row.readBool('live'), throwsA(isA<SqlxError>()));
  });

  test('a column that is neither DateTime nor text is not a date', () {
    final row = PostgresRow(_row(<String, Object?>{'at': 7}));

    expect(() => row.readDateTime('at'), throwsA(isA<SqlxError>()));
  });

  test('a null timestamp is reported when a value is required', () {
    final row = PostgresRow(_row(<String, Object?>{'at': null}));

    expect(row.readDateTimeNullable('at'), isNull);
    expect(() => row.readDateTime('at'), throwsA(isA<SqlxError>()));
  });

  group('column index', () {
    test('an unnamed column is reachable by its position', () {
      // `SELECT 1` names nothing, and the driver's own column map keys such a
      // column `[0]`. Reading it that way has to keep working.
      final schema = pg.ResultSchema(<pg.ResultSchemaColumn>[
        pg.ResultSchemaColumn(typeOid: 0, type: pg.Type.unspecified),
      ]);
      final row = PostgresRow(
        pg.ResultRow(schema: schema, values: <Object?>[7]),
      );

      expect(row.read<int>('[0]'), 7);
      expect(row.readIndex<int>(0), 7);
    });

    test('a name selected twice resolves to the last of them', () {
      final schema = pg.ResultSchema(<pg.ResultSchemaColumn>[
        pg.ResultSchemaColumn(
            typeOid: 0, type: pg.Type.unspecified, columnName: 'id'),
        pg.ResultSchemaColumn(
            typeOid: 0, type: pg.Type.unspecified, columnName: 'id'),
      ]);
      final row = PostgresRow(
        pg.ResultRow(schema: schema, values: <Object?>[1, 2]),
      );

      expect(row.read<int>('id'), 2);
    });

    test('rows sharing one schema read the same names', () {
      // What `_rows` relies on: one index built per result, used by every row.
      final schema = pg.ResultSchema(<pg.ResultSchemaColumn>[
        pg.ResultSchemaColumn(
            typeOid: 0, type: pg.Type.unspecified, columnName: 'name'),
      ]);
      final index = postgresColumnIndex(schema);
      final rows = <PostgresRow>[
        for (final name in <String>['Ada', 'Grace'])
          PostgresRow(pg.ResultRow(schema: schema, values: <Object?>[name])),
      ];

      expect(index, <String, int>{'name': 0});
      expect(<String>[for (final row in rows) row.read<String>('name')],
          <String>['Ada', 'Grace']);
    });
  });
}
