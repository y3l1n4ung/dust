import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

import 'support/user_name.dart';

void main() {
  test('sqlite pool queries and transactions', () async {
    final pool = SqlitePool.open(':memory:', migrations: const <String, String>{
      '0001.sql': '''
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT
);
''',
    });
    addTearDown(() async {
      await pool.close();
    });

    final inserted = await queryExecute(
      r'INSERT INTO users (id, name, active, created_at) VALUES (?, ?, ?, ?)',
      [1, 'Ada', 1, '2026-01-01T10:00:00+06:30'],
    ).execute(pool);
    expect(inserted.rowsAffected, 1);
    expect(inserted.lastInsertId, 1);

    final rows = await queryRaw(
      r'SELECT id, name, active, created_at FROM users WHERE id = ?',
      [1],
    ).fetch(pool);
    expect(rows.single.read<int>('id'), 1);
    expect(rows.single.read<String>('name'), 'Ada');
    expect(rows.single.read<double>('id'), 1.0);
    expect(rows.single.read<num>('id'), 1);
    expect(rows.single.readBool('active'), isTrue);
    expect(rows.single.readDateTime('created_at'), DateTime.utc(2026, 1, 1, 3, 30));

    final txResult = await pool.transaction((tx) async {
      await queryExecute(r'UPDATE users SET name = ? WHERE id = ?', [
        'Grace',
        1,
      ]).execute(tx);
      return const Ok<void, SqlxError>(null);
    });
    expect(txResult, isA<Ok<void, SqlxError>>());

    final updated = await queryRaw(r'SELECT name FROM users WHERE id = ?', [
      1,
    ]).fetch(pool);
    expect(updated.single.read<String>('name'), 'Grace');
  });

  test('sqlite typed fetch methods enforce cardinality and decode errors', () async {
    final pool = Sqlite3Driver.open(':memory:', migrations: const <String, String>{
      '0001.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, active TEXT);',
    });
    addTearDown(() async {
      await pool.close();
    });

    await queryExecute(r'INSERT INTO users (id, name, active) VALUES (?, ?, ?)', [
      1,
      'Ada',
      'true',
    ]).execute(pool);
    await queryExecute(r'INSERT INTO users (id, name, active) VALUES (?, ?, ?)', [
      2,
      null,
      'maybe',
    ]).execute(pool);

    final optional = await pool.fetchOptional<UserName>(
      r'SELECT id, name FROM users WHERE id = ?',
      [1],
      UserName.fromRow,
    );
    expect(optional.match(ok: (value) => value?.name, err: (_) => 'err'), 'Ada');

    final missingOptional = await pool.fetchOptional<UserName>(
      r'SELECT id, name FROM users WHERE id = ?',
      [99],
      UserName.fromRow,
    );
    expect(missingOptional.match(ok: (value) => value, err: (_) => 'err'), isNull);

    final all = await pool.fetchAll<UserName>(
      r'SELECT id, name FROM users WHERE id = ?',
      [1],
      UserName.fromRow,
    );
    expect(all.match(ok: (value) => value.single.name, err: (_) => 'err'), 'Ada');

    final none = await pool.fetchOne<UserName>(
      r'SELECT id, name FROM users WHERE id = ?',
      [99],
      UserName.fromRow,
    );
    expect(none, isA<Err<UserName, SqlxError>>());

    final tooMany = await pool.fetchOne<UserName>(
      r'SELECT id, name FROM users ORDER BY id',
      const [],
      UserName.fromRow,
    );
    expect(tooMany, isA<Err<UserName, SqlxError>>());

    final decode = await pool.fetchOne<UserName>(
      r'SELECT id, name FROM users WHERE id = ?',
      [2],
      UserName.fromRow,
    );
    expect(decode, isA<Err<UserName, SqlxError>>());

    final boolRows = await queryRaw(r'SELECT active FROM users WHERE id = ?', [1]).fetch(pool);
    expect(boolRows.single.readBool('active'), isTrue);
    expect(boolRows.single.readBoolOrNull('missing'), isNull);

    final badBoolRows = await queryRaw(r'SELECT active FROM users WHERE id = ?', [2]).fetch(pool);
    expect(() => badBoolRows.single.readBool('active'), throwsA(isA<SqlxDecodeError>()));

    final nullNameRows = await queryRaw(r'SELECT name FROM users WHERE id = ?', [2]).fetch(pool);
    expect(() => nullNameRows.single.read<String>('name'), throwsA(isA<SqlxDecodeError>()));

    final normalMapperError = await pool.fetchAll<UserName>(
      r'SELECT id, name FROM users WHERE id = ?',
      [1],
      (_) => throw StateError('mapper failed'),
    );
    expect(normalMapperError, isA<Err<List<UserName>, SqlxError>>());

    final sqlxMapperError = await pool.fetchAll<UserName>(
      r'SELECT id, name FROM users WHERE id = ?',
      [1],
      (_) => throw SqlxError.decode('mapper failed'),
    );
    expect(sqlxMapperError, isA<Err<List<UserName>, SqlxError>>());

    final oneMapperError = await pool.fetchOne<UserName>(
      r'SELECT id, name FROM users WHERE id = ?',
      [1],
      (_) => throw StateError('mapper failed'),
    );
    expect(oneMapperError, isA<Err<UserName, SqlxError>>());

    final optionalQueryError = await pool.fetchOptional<UserName>(
      'SELECT * FROM missing_table',
      const [],
      UserName.fromRow,
    );
    final allQueryError = await pool.fetchAll<UserName>(
      'SELECT * FROM missing_table',
      const [],
      UserName.fromRow,
    );
    final oneQueryError = await pool.fetchOne<UserName>(
      'SELECT * FROM missing_table',
      const [],
      UserName.fromRow,
    );
    expect(optionalQueryError, isA<Err<UserName?, SqlxError>>());
    expect(allQueryError, isA<Err<List<UserName>, SqlxError>>());
    expect(oneQueryError, isA<Err<UserName, SqlxError>>());
  });

  test('sqlite scalar fetch handles nullability, too many rows, and closed driver', () async {
    final pool = Sqlite3Driver.open(':memory:', migrations: const <String, String>{
      '0001.sql': 'CREATE TABLE numbers (value INTEGER);',
    });

    final emptyNullable = await pool.fetchScalar<int?>('SELECT value FROM numbers', const []);
    expect(emptyNullable.match(ok: (value) => value, err: (_) => -1), isNull);

    final emptyRequired = await pool.fetchScalar<int>('SELECT value FROM numbers', const []);
    expect(emptyRequired, isA<Err<int, SqlxError>>());

    await queryExecute(r'INSERT INTO numbers (value) VALUES (?)', [1]).execute(pool);
    await queryExecute(r'INSERT INTO numbers (value) VALUES (?)', [2]).execute(pool);

    final tooMany = await pool.fetchScalar<int>('SELECT value FROM numbers ORDER BY value', const []);
    expect(tooMany, isA<Err<int, SqlxError>>());

    final first = await pool.fetchScalar<int>('SELECT value FROM numbers WHERE value = 1', const []);
    final nullableValue = await pool.fetchScalar<int?>('SELECT value FROM numbers WHERE value = 1', const []);
    expect(first.match(ok: (value) => value, err: (_) => -1), 1);
    expect(nullableValue.match(ok: (value) => value, err: (_) => -1), 1);

    await queryExecute(r'INSERT INTO numbers (value) VALUES (?)', [null]).execute(pool);
    final nullRequired = await pool.fetchScalar<int>('SELECT value FROM numbers WHERE value IS NULL', const []);
    expect(nullRequired, isA<Err<int, SqlxError>>());

    final wrongType = await pool.fetchScalar<int>(
      'SELECT CAST(value AS TEXT) FROM numbers WHERE value = 1',
      const [],
    );
    expect(wrongType, isA<Err<int, SqlxError>>());

    final queryError = await pool.fetchScalar<int>('SELECT value FROM missing_table', const []);
    expect(queryError, isA<Err<int, SqlxError>>());

    await pool.close();
    await pool.close();

    final afterClose = await pool.fetchScalar<int>('SELECT value FROM numbers', const []);
    expect(afterClose, isA<Err<int, SqlxError>>());
  });
}
