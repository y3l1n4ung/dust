import 'package:dust_db/dust_db.dart';
import 'package:test/test.dart';

final class StatusFromInt implements SqlxTryFrom<String, int> {
  const StatusFromInt();

  @override
  String decode(int value) => value.toString();
}

void main() {
  test('annotations preserve sqlx-style options', () {
    const db = DustDb(driver: Driver.sqflite, migrations: 'migrations');
    const query = Query('SELECT 1');
    const config = Sqlx(
      rename: 'display_name',
      renameAll: SqlxRename.snakeCase,
      flatten: true,
      defaultValue: '',
      skip: true,
      json: true,
      tryFrom: StatusFromInt(),
    );

    expect(db.driver, Driver.sqflite);
    expect(db.migrations, 'migrations');
    expect(query.sql, 'SELECT 1');
    expect(config.rename, 'display_name');
    expect(config.renameAll, SqlxRename.snakeCase);
    expect(config.flatten, isTrue);
    expect(config.defaultValue, '');
    expect(config.skip, isTrue);
    expect(config.json, isTrue);
    expect(config.tryFrom, isA<StatusFromInt>());
  });
}
