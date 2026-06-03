import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

final class StatusFromInt implements SqlxTryFrom<String, int> {
  const StatusFromInt();

  @override
  String decode(int value) => value.toString();
}

void main() {
  test('annotations preserve sqlx-style options', () {
    const db = SqlxDatabase(
      type: SqlxDatabaseType.sqlite,
      migrations: './migrations',
    );
    const dao = SqlxDao();
    const query = Query(r'SELECT * FROM users WHERE id = $1');
    const config = Sqlx(
      rename: 'display_name',
      renameAll: SqlxRename.snakeCase,
      flatten: true,
      defaultValue: '',
      skip: true,
      json: true,
      tryFrom: StatusFromInt(),
    );
    const fromRow = FromRow();

    expect(db.type, SqlxDatabaseType.sqlite);
    expect(db.migrations, './migrations');
    expect(dao, isA<SqlxDao>());
    expect(query.sql, r'SELECT * FROM users WHERE id = $1');
    expect(config.rename, 'display_name');
    expect(config.renameAll, SqlxRename.snakeCase);
    expect(config.flatten, isTrue);
    expect(config.defaultValue, '');
    expect(config.skip, isTrue);
    expect(config.json, isTrue);
    expect(config.tryFrom, isA<StatusFromInt>());
    expect(fromRow, isA<DeriveTrait>());
  });
}
