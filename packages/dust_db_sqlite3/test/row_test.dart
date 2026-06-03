import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

void main() {
  test('sqlite row typed readers reject null and unsupported values', () async {
    final pool = SqlitePool.open(':memory:');
    addTearDown(() async {
      await pool.close();
    });

    final rows = await queryRaw(
      "SELECT NULL AS value, 'false' AS false_text, '0' AS zero_text, "
      "x'01' AS blob_value, 'not-a-date' AS bad_date",
      const [],
    ).fetch(pool);
    final row = rows.single;

    expect(row.readBool('false_text'), isFalse);
    expect(row.readBool('zero_text'), isFalse);
    expect(row.readDateTimeOrNull('value'), isNull);
    expect(row.readIndexOrNull<int>(0), isNull);
    expect(() => row.readIndex<int>(0), throwsA(isA<SqlxDecodeError>()));
    expect(() => row.readBool('value'), throwsA(isA<SqlxDecodeError>()));
    expect(() => row.readBool('blob_value'), throwsA(isA<SqlxDecodeError>()));
    expect(() => row.readDateTime('value'), throwsA(isA<SqlxDecodeError>()));
    expect(() => row.readDateTime('bad_date'), throwsA(isA<FormatException>()));
    expect(
      () => row.readDateTime('blob_value'),
      throwsA(isA<SqlxDecodeError>()),
    );
  });
}
