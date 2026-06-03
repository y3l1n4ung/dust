import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

void main() {
  test('sqlite migrations run once and apply upgrades in name order', () async {
    final directory = await Directory.systemTemp.createTemp('dust_sqlite_');
    addTearDown(() async {
      await directory.delete(recursive: true);
    });
    final path = '${directory.path}/app.db';

    final first = SqlitePool.open(path, migrations: const <String, String>{
      '0001_create.sql': '''
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);
INSERT INTO users (id, name) VALUES (1, 'Ada');
''',
    });
    await first.close();

    final upgraded = SqlitePool.open(path, migrations: const <String, String>{
      '0001_create.sql': '''
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);
INSERT INTO users (id, name) VALUES (1, 'Ada');
''',
      '0002_upgrade.sql': '''
ALTER TABLE users ADD COLUMN active INTEGER NOT NULL DEFAULT 1;
INSERT INTO users (id, name, active) VALUES (2, 'Grace', 0);
''',
    });
    addTearDown(() async {
      await upgraded.close();
    });

    final users = await queryRaw(
      'SELECT id, name, active FROM users ORDER BY id',
      const [],
    ).fetch(upgraded);
    expect(users, hasLength(2));
    expect(users[0].read<String>('name'), 'Ada');
    expect(users[0].readBool('active'), isTrue);
    expect(users[1].read<String>('name'), 'Grace');
    expect(users[1].readBool('active'), isFalse);

    final migrations = await queryRaw(
      'SELECT name FROM __dust_schema_migrations ORDER BY name',
      const [],
    ).fetch(upgraded);
    expect(
      migrations.map((row) => row.read<String>('name')),
      <String>['0001_create.sql', '0002_upgrade.sql'],
    );
  });

  test('sqlite migration failure rolls back the open transaction', () async {
    final directory = await Directory.systemTemp.createTemp('dust_sqlite_');
    addTearDown(() async {
      await directory.delete(recursive: true);
    });
    final path = '${directory.path}/app.db';

    expect(
      () => SqlitePool.open(path, migrations: const <String, String>{
        '0001_create.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY);',
        '0002_broken.sql': 'INSERT INTO missing_table (id) VALUES (1);',
      }),
      throwsA(isA<Exception>()),
    );

    final reopened = SqlitePool.open(path, migrations: const <String, String>{
      '0001_create.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY);',
    });
    addTearDown(() async {
      await reopened.close();
    });

    final migrations = await queryRaw(
      'SELECT name FROM __dust_schema_migrations ORDER BY name',
      const [],
    ).fetch(reopened);
    expect(
      migrations.map((row) => row.read<String>('name')),
      <String>['0001_create.sql'],
    );
  });
}
