import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

void main() {
  test('sqlite connect options open in-memory database with pragmas', () async {
    final db = Sqlite3Driver.connect(
      const SqliteConnectOptions.memory(
        foreignKeys: true,
        busyTimeout: Duration(milliseconds: 250),
        journalMode: SqliteJournalMode.memory,
        synchronous: SqliteSynchronousMode.normal,
        pragmas: <String, Object>{'cache_size': -2000},
      ),
      migrations: const <String, String>{
        '0001.sql': '''
CREATE TABLE parents (id INTEGER PRIMARY KEY);
CREATE TABLE children (
  id INTEGER PRIMARY KEY,
  parent_id INTEGER NOT NULL REFERENCES parents(id)
);
''',
      },
    );
    addTearDown(() async {
      await db.close();
    });

    expect(_pragmaInt(db, 'foreign_keys'), 1);
    expect(_pragmaInt(db, 'busy_timeout'), 250);
    expect(_pragmaInt(db, 'cache_size'), -2000);

    final invalidChild = await db.execute(
      'INSERT INTO children (id, parent_id) VALUES (?, ?)',
      const [1, 99],
    );
    expect(invalidChild, isA<Err<ExecResult, SqlxError>>());
  });

  test('sqlite connect options support path and read-only databases', () async {
    final directory = await Directory.systemTemp.createTemp('dust_sqlite_');
    addTearDown(() async {
      await directory.delete(recursive: true);
    });
    final path = '${directory.path}/app.db';

    expect(
      () => Sqlite3Driver.open(
        path,
        options: const SqliteConnectOptions(createIfMissing: false),
      ),
      throwsA(isA<SqlxDriverError>()),
    );

    final writable = Sqlite3Driver.connect(
      SqliteConnectOptions.path(path, foreignKeys: true),
      migrations: const <String, String>{
        '0001.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);',
      },
    );
    await queryExecute('INSERT INTO users (id, name) VALUES (?, ?)', const [
      1,
      'Ada',
    ]).execute(writable);
    await writable.close();

    final readOnly = Sqlite3Driver.connect(SqliteConnectOptions.readOnly(path));
    addTearDown(() async {
      await readOnly.close();
    });

    final users = await queryRaw(
      'SELECT name FROM users ORDER BY id',
      const [],
    ).fetch(readOnly);
    expect(users.single.read<String>('name'), 'Ada');

    final write = await readOnly.execute(
      'INSERT INTO users (id, name) VALUES (?, ?)',
      const [2, 'Grace'],
    );
    expect(write, isA<Err<ExecResult, SqlxError>>());
  });

  test('sqlite connect options report invalid combinations clearly', () async {
    final directory = await Directory.systemTemp.createTemp('dust_sqlite_');
    addTearDown(() async {
      await directory.delete(recursive: true);
    });
    final path = '${directory.path}/app.db';

    expect(
      () => Sqlite3Driver.connect(
        SqliteConnectOptions.path(path, readOnly: true),
      ),
      throwsA(isA<SqlxDriverError>()),
    );
    expect(
      () => Sqlite3Driver.connect(
        SqliteConnectOptions.readOnly(path),
        migrations: const <String, String>{
          '0001.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY);',
        },
      ),
      throwsA(isA<SqlxDriverError>()),
    );
    expect(
      () => Sqlite3Driver.connect(
        const SqliteConnectOptions.memory(
          pragmas: <String, Object>{'bad-name': 1},
        ),
      ),
      throwsA(isA<SqlxDriverError>()),
    );
  });
}

int _pragmaInt(Sqlite3Driver db, String name) {
  return db.database.select('PRAGMA $name').single.columnAt(0) as int;
}
