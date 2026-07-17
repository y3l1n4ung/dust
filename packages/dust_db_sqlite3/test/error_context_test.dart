import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

void main() {
  test('query and decode errors include SQLite context', () async {
    final db = Sqlite3Driver.open(
      ':memory:',
      migrations: const <String, String>{
        '0001.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);',
      },
    );
    addTearDown(() async {
      await db.close();
    });

    final missingTable = _errorOf(
      await db.fetchOne<_User>(
        'SELECT id, name FROM missing_table',
        const [],
        _User.fromRow,
      ),
    );
    expect(missingTable.category, SqlxErrorCategory.query);
    expect(missingTable.driver, Driver.sqlite3);
    expect(missingTable.operation, 'SELECT id, name FROM missing_table');

    await db.execute(
      'INSERT INTO users (id, name) VALUES (?, ?)',
      const [1, null],
    );
    final decode = _errorOf(
      await db.fetchOne<_User>(
        'SELECT id, name FROM users WHERE id = ?',
        const [1],
        _User.fromRow,
      ),
    );
    expect(decode.category, SqlxErrorCategory.decode);
    expect(decode.driver, Driver.sqlite3);
    expect(decode.operation, 'read:name');
  });

  test('cardinality and closed connection errors include context', () async {
    final db = Sqlite3Driver.open(
      ':memory:',
      migrations: const <String, String>{
        '0001.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);',
      },
    );

    final noRows = _errorOf(
      await db.fetchOne<_User>(
        'SELECT id, name FROM users',
        const [],
        _User.fromRow,
      ),
    );
    expect(noRows.category, SqlxErrorCategory.cardinality);
    expect(noRows.driver, Driver.sqlite3);
    expect(noRows.operation, 'SELECT id, name FROM users');

    await db.close();
    final closed = _errorOf(
      await db.fetchScalar<int>('SELECT COUNT(*) FROM users', const []),
    );
    expect(closed.category, SqlxErrorCategory.connection);
    expect(closed.driver, Driver.sqlite3);
    expect(closed.operation, 'checkOpen');
  });

  test('migration failures include migration context', () async {
    final directory = await Directory.systemTemp.createTemp('dust_sqlite_');
    addTearDown(() async {
      await directory.delete(recursive: true);
    });

    expect(
      () => SqlitePool.open(
        '${directory.path}/app.db',
        migrations: const <String, String>{
          '0001_create.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY);',
          '0002_broken.sql': 'INSERT INTO missing_table (id) VALUES (1);',
        },
      ),
      throwsA(
        isA<SqlxDriverError>()
            .having(
              (error) => error.category,
              'category',
              SqlxErrorCategory.migration,
            )
            .having((error) => error.driver, 'driver', Driver.sqlite3)
            .having(
              (error) => error.operation,
              'operation',
              '0002_broken.sql',
            ),
      ),
    );
  });
}

SqlxError _errorOf<T>(Result<T, SqlxError> result) {
  return result.match(
    ok: (_) => throw StateError('expected Err'),
    err: (error) => error,
  );
}

final class _User {
  const _User({required this.id, required this.name});

  final int id;
  final String name;

  static _User fromRow(Row row) {
    return _User(id: row.read<int>('id'), name: row.read<String>('name'));
  }
}
