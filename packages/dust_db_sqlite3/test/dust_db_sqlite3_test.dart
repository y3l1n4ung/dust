import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

void main() {
  test('sqlite pool queries and transactions', () async {
    final pool = SqlitePool.open(
      ':memory:',
      migrations: const <String, String>{
        '0001.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);',
      },
    );
    addTearDown(() async {
      await pool.close();
    });

    final inserted = await queryExecute(
      r'INSERT INTO users (id, name) VALUES ($1, $2)',
      [1, 'Ada'],
    ).execute(pool);
    expect(inserted.rowsAffected, 1);

    final rows = await queryRaw(
      r'SELECT id, name FROM users WHERE id = $1',
      [1],
    ).fetch(pool);
    expect(rows.single.read<int>('id'), 1);
    expect(rows.single.read<String>('name'), 'Ada');

    final txResult = await pool.transaction((tx) async {
      await queryExecute(r'UPDATE users SET name = $1 WHERE id = $2', [
        'Grace',
        1,
      ]).execute(tx);
      return const Ok<void, SqlxError>(null);
    });
    expect(txResult, isA<Ok<void, SqlxError>>());

    final updated = await queryRaw(r'SELECT name FROM users WHERE id = $1', [1]).fetch(pool);
    expect(updated.single.read<String>('name'), 'Grace');
  });

  test('sqlite transaction rolls back on error', () async {
    final pool = SqlitePool.open(
      ':memory:',
      migrations: const <String, String>{
        '0001.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);',
      },
    );
    addTearDown(() async {
      await pool.close();
    });

    final result = await pool.transaction((tx) async {
      await queryExecute(r'INSERT INTO users (id, name) VALUES ($1, $2)', [
        1,
        'Ada',
      ]).execute(tx);
      throw StateError('boom');
    });
    expect(result, isA<Err<void, SqlxError>>());

    final rows = await queryRaw('SELECT id FROM users', []).fetch(pool);
    expect(rows, isEmpty);
  });
}
