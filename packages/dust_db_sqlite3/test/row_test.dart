import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:sqlite3/sqlite3.dart' as sqlite;
import 'package:test/test.dart';

void main() {
  test('sqlite row reads every supported conversion', () async {
    final pool = SqlitePool.open(':memory:');
    addTearDown(() async {
      await pool.close();
    });

    final rows = await queryRaw(
      "SELECT 1 AS id, 2 AS score, 3.5 AS ratio, 'Ada' AS name, "
      "1 AS one_bool, 0 AS zero_bool, 'true' AS true_text, "
      "'false' AS false_text, '2026-01-01T10:00:00+06:30' AS created_at, "
      "NULL AS empty_value",
      const [],
    ).fetch(pool);
    final row = rows.single;

    expect(row, isA<Sqlite3Row>());
    expect(row.read<int>('id'), 1);
    expect(row.read<double>('score'), 2.0);
    expect(row.read<num>('ratio'), 3.5);
    expect(row.read<String>('name'), 'Ada');
    expect(row.readBool('one_bool'), isTrue);
    expect(row.readBool('zero_bool'), isFalse);
    expect(row.readBool('true_text'), isTrue);
    expect(row.readBool('false_text'), isFalse);
    expect(row.readDateTime('created_at'), DateTime.utc(2026, 1, 1, 3, 30));
    expect(row.readNullable<Object?>('empty_value'), isNull);
    expect(row.readIndex<int>(0), 1);
    expect(row.readIndexNullable<Object?>(99), isNull);
  });

  test(
    'sqlite row reports null and unsupported values as decode errors',
    () async {
      final pool = SqlitePool.open(':memory:');
      addTearDown(() async {
        await pool.close();
      });

      final rows = await queryRaw(
        "SELECT NULL AS value, 'maybe' AS bad_bool, x'01' AS blob_value, "
        "'not-a-date' AS bad_date",
        const [],
      ).fetch(pool);
      final row = rows.single;

      expect(row.readDateTimeNullable('value'), isNull);
      expect(row.readIndexNullable<int>(0), isNull);
      expect(() => row.readIndex<int>(0), throwsA(isA<SqlxDecodeError>()));
      expect(() => row.readBool('value'), throwsA(isA<SqlxDecodeError>()));
      expect(() => row.readBool('bad_bool'), throwsA(isA<SqlxDecodeError>()));
      expect(() => row.readBool('blob_value'), throwsA(isA<SqlxDecodeError>()));
      expect(() => row.readDateTime('value'), throwsA(isA<SqlxDecodeError>()));
      expect(
        () => row.readDateTime('bad_date'),
        throwsA(isA<SqlxDecodeError>()),
      );
      expect(
        () => row.readDateTime('blob_value'),
        throwsA(isA<SqlxDecodeError>()),
      );
      expect(
        () => row.read<int>('blob_value'),
        throwsA(isA<SqlxDecodeError>()),
      );
    },
  );

  test('Sqlite3Row wraps direct sqlite3 result rows', () {
    final database = sqlite.sqlite3.openInMemory();
    addTearDown(database.close);

    database.execute('CREATE TABLE users (id INTEGER, active INTEGER)');
    database.execute('INSERT INTO users (id, active) VALUES (7, 1)');

    final sqliteRow = database.select('SELECT id, active FROM users').single;
    final row = Sqlite3Row(sqliteRow);

    expect(row.read<int>('id'), 7);
    expect(row.readBool('active'), isTrue);
  });
}
