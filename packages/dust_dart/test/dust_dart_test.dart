import 'package:dust_dart/db.dart' as db;
import 'package:dust_dart/dust_dart.dart';
import 'package:dust_dart/http.dart' as http;
import 'package:test/test.dart';

void main() {
  test('exports Dart-only Dust APIs', () {
    const derive = Derive([ToString(), Eq(), CopyWith()]);
    const serde = SerDe(renameAll: SerDeRename.snakeCase);
    const database = SqlxDatabase(type: SqlxDatabaseType.sqlite);
    const dao = SqlxDao();
    const query = Query('SELECT 1');

    expect(derive.traits, hasLength(3));
    expect(serde.renameAll, SerDeRename.snakeCase);
    expect(database.type, SqlxDatabaseType.sqlite);
    expect(dao, isA<SqlxDao>());
    expect(query.sql, 'SELECT 1');
  });

  test(
    'keeps DB and HTTP Query annotations available through scoped libraries',
    () {
      const sqlQuery = db.Query('SELECT 1');
      const httpQuery = http.Query('search');

      expect(sqlQuery.sql, 'SELECT 1');
      expect(httpQuery.name, 'search');
    },
  );

  test('exports runtime Result API', () {
    const result = Ok<int, SqlxError>(1);
    expect(result.match(ok: (value) => value, err: (_) => 0), 1);
  });
}
