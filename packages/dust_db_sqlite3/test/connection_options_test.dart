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

  group('SqliteConnectOptions constructors', () {
    test('default constructor uses write mode without a fixed path', () {
      const options = SqliteConnectOptions();
      expect(options.path, isNull);
      expect(options.inMemory, isFalse);
      expect(options.createIfMissing, isTrue);
      expect(options.readOnly, isFalse);
      expect(options.pragmas, isEmpty);
    });

    test('path constructor fixes the path and keeps write defaults', () {
      const options = SqliteConnectOptions.path('app.db', foreignKeys: true);
      expect(options.path, 'app.db');
      expect(options.inMemory, isFalse);
      expect(options.createIfMissing, isTrue);
      expect(options.readOnly, isFalse);
      expect(options.foreignKeys, isTrue);
    });

    test('readOnly constructor disables create-if-missing and enables read-only', () {
      const options = SqliteConnectOptions.readOnly('app.db');
      expect(options.path, 'app.db');
      expect(options.inMemory, isFalse);
      expect(options.readOnly, isTrue);
      expect(options.createIfMissing, isFalse);
    });

    test('memory constructor has no path and uses write mode', () {
      const options = SqliteConnectOptions.memory();
      expect(options.path, isNull);
      expect(options.inMemory, isTrue);
      expect(options.createIfMissing, isTrue);
      expect(options.readOnly, isFalse);
    });
  });

  test('connect requires a path when not using an in-memory database', () {
    expect(
      () => Sqlite3Driver.connect(const SqliteConnectOptions()),
      throwsA(
        isA<SqlxDriverError>().having(
          (error) => error.message,
          'message',
          contains('SQLite path is required'),
        ),
      ),
    );
  });

  test('open rejects explicit options with a path that conflicts with the open path', () {
    expect(
      () => Sqlite3Driver.open(
        'a.db',
        options: SqliteConnectOptions.path('b.db'),
      ),
      throwsA(
        isA<SqlxDriverError>().having(
          (error) => error.message,
          'message',
          contains('does not match'),
        ),
      ),
    );
  });

  test('connect rejects a negative busyTimeout before opening the database', () {
    expect(
      () => Sqlite3Driver.connect(
        const SqliteConnectOptions.memory(
          busyTimeout: Duration(milliseconds: -1),
        ),
      ),
      throwsA(
        isA<SqlxDriverError>().having(
          (error) => error.message,
          'message',
          contains('busyTimeout must not be negative'),
        ),
      ),
    );
  });

  test('connect rejects pragma values that are not bool, num, or String', () {
    expect(
      () => Sqlite3Driver.connect(
        const SqliteConnectOptions.memory(
          pragmas: <String, Object>{'bad_value': <int>[1]},
        ),
      ),
      throwsA(
        isA<SqlxDriverError>().having(
          (error) => error.message,
          'message',
          contains('Invalid SQLite pragma value'),
        ),
      ),
    );
  });

  test('connect applies custom string pragma values', () async {
    final db = Sqlite3Driver.connect(
      const SqliteConnectOptions.memory(
        pragmas: <String, Object>{'temp_store': 'FILE'},
      ),
    );
    addTearDown(() async {
      await db.close();
    });

    expect(db.database.select('PRAGMA temp_store').single.columnAt(0), 1);
  });

  test('connect escapes embedded quotes in custom string pragma values', () async {
    final db = Sqlite3Driver.connect(
      const SqliteConnectOptions.memory(
        pragmas: <String, Object>{'dust_test_marker': "O'Brien"},
      ),
    );
    addTearDown(() async {
      await db.close();
    });

    // `dust_test_marker` is not a real SQLite pragma; SQLite treats unknown
    // pragmas as no-ops rather than errors. This confirms the generated
    // `PRAGMA dust_test_marker = 'O''Brien'` statement is syntactically
    // valid and does not throw.
    expect(db.database.select('SELECT 1').single.columnAt(0), 1);
  });

  test('connect applies the requested journal mode for file-based databases', () async {
    final directory = await Directory.systemTemp.createTemp('dust_sqlite_');
    addTearDown(() async {
      await directory.delete(recursive: true);
    });

    const expected = <SqliteJournalMode, String>{
      SqliteJournalMode.delete: 'delete',
      SqliteJournalMode.truncate: 'truncate',
      SqliteJournalMode.persist: 'persist',
      SqliteJournalMode.memory: 'memory',
      SqliteJournalMode.wal: 'wal',
      SqliteJournalMode.off: 'off',
    };

    for (final entry in expected.entries) {
      final path = '${directory.path}/${entry.key.name}.db';
      final db = Sqlite3Driver.connect(
        SqliteConnectOptions.path(path, journalMode: entry.key),
      );
      expect(
        db.database.select('PRAGMA journal_mode').single.columnAt(0),
        entry.value,
        reason: 'journal mode ${entry.key}',
      );
      await db.close();
    }
  });

  test('connect applies the requested synchronous mode', () async {
    final directory = await Directory.systemTemp.createTemp('dust_sqlite_');
    addTearDown(() async {
      await directory.delete(recursive: true);
    });

    const expected = <SqliteSynchronousMode, int>{
      SqliteSynchronousMode.off: 0,
      SqliteSynchronousMode.normal: 1,
      SqliteSynchronousMode.full: 2,
      SqliteSynchronousMode.extra: 3,
    };

    for (final entry in expected.entries) {
      final path = '${directory.path}/${entry.key.name}.db';
      final db = Sqlite3Driver.connect(
        SqliteConnectOptions.path(path, synchronous: entry.key),
      );
      expect(
        db.database.select('PRAGMA synchronous').single.columnAt(0),
        entry.value,
        reason: 'synchronous mode ${entry.key}',
      );
      await db.close();
    }
  });

  test('open reports a descriptive error when the underlying sqlite open fails', () async {
    final directory = await Directory.systemTemp.createTemp('dust_sqlite_');
    addTearDown(() async {
      await directory.delete(recursive: true);
    });
    final path = '${directory.path}/missing.db';

    expect(
      () => Sqlite3Driver.open(
        path,
        options: const SqliteConnectOptions(createIfMissing: false),
      ),
      throwsA(
        isA<SqlxDriverError>().having(
          (error) => error.message,
          'message',
          contains('SQLite database open failed'),
        ),
      ),
    );
  });
}

int _pragmaInt(Sqlite3Driver db, String name) {
  return db.database.select('PRAGMA $name').single.columnAt(0) as int;
}
