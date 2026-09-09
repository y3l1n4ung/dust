import 'dart:typed_data';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

import 'support/expect_ok.dart';
import 'support/unsafe_rows.dart';

/// Opens an in-memory pool holding one table with a text and a blob column.
SqlitePool _pool() {
  return SqlitePool.open(
    ':memory:',
    migrations: const <String, String>{
      '0001.sql': '''
CREATE TABLE items (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  payload BLOB
);
''',
    },
  );
}

void main() {
  test('a List argument binds as JSON text for the json_each idiom', () async {
    final pool = _pool();
    addTearDown(() async {
      await pool.close();
    });
    for (final row in <List<Object?>>[
      [1, 'one'],
      [2, 'two'],
      [3, 'three'],
    ]) {
      expectOk(
        await queryExecute(
          r'INSERT INTO items (id, name) VALUES (?, ?)',
          row,
        ).execute(pool),
      );
    }

    // Constant SQL, one placeholder, one bound value — the shape `describe`
    // accepts, and the reason no caller needs dynamic `IN (?, ?, ?)`.
    final rows = await unsafeRows(
        pool,
        r'SELECT name FROM items '
        r'WHERE id IN (SELECT value FROM json_each(?)) ORDER BY id',
        [
          const <int>[1, 3],
        ]);

    expect(
      rows.map((row) => row.read<String>('name')),
      <String>['one', 'three'],
    );
  });

  test('an empty List argument selects nothing rather than failing', () async {
    final pool = _pool();
    addTearDown(() async {
      await pool.close();
    });
    expectOk(
      await queryExecute(
        r'INSERT INTO items (id, name) VALUES (?, ?)',
        [1, 'one'],
      ).execute(pool),
    );

    final rows = await unsafeRows(
        pool,
        r'SELECT name FROM items WHERE id IN (SELECT value FROM json_each(?))',
        [const <int>[]]);

    expect(rows, isEmpty);
  });

  test('a Uint8List still binds as a BLOB rather than as JSON', () async {
    final pool = _pool();
    addTearDown(() async {
      await pool.close();
    });
    final payload = Uint8List.fromList(<int>[1, 2, 3]);

    expectOk(
      await queryExecute(
        r'INSERT INTO items (id, name, payload) VALUES (?, ?, ?)',
        [1, 'one', payload],
      ).execute(pool),
    );

    // A JSON-encoded payload would come back as the five characters `[1,2,3]`.
    final stored = (await unsafeRows(
      pool,
      r'SELECT length(payload) AS size, typeof(payload) AS kind FROM items',
    ))
        .single;

    expect(stored.read<int>('size'), 3);
    expect(stored.read<String>('kind'), 'blob');
  });

  test('a List holding a value JSON cannot carry reports it', () async {
    final pool = _pool();
    addTearDown(() async {
      await pool.close();
    });

    final result = await queryExecute(
      r'INSERT INTO items (id, name) VALUES (?, ?)',
      [
        1,
        <Object?>[Object()],
      ],
    ).execute(pool);

    expect(result.isErr, isTrue);
    expect(
      result.match(ok: (_) => '', err: (error) => error.message),
      contains('JSON cannot represent'),
    );
  });
}
