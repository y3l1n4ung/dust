import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

import 'support/postgres.dart';

/// Statement reuse, which is most of what a small query costs over a socket.
///
/// The cache is private, so these assert on what it is for: the same SQL run
/// many times keeps working, a schema change under a held statement recovers
/// rather than failing every call after it, and closing leaves nothing behind.
void main() {
  if (databaseUrl == null) {
    test('statement cache', () {}, skip: skipWithoutDatabase);
    return;
  }

  const table = 'cached_items';

  Future<PostgresDriver> driver() => tableFor(
        'CREATE TABLE $table (id BIGINT PRIMARY KEY, name TEXT NOT NULL)',
        table,
      );

  test('the same statement run many times keeps its parameters straight',
      () async {
    final db = await driver();

    for (var i = 1; i <= 50; i++) {
      expectOk(await db.execute(
        'INSERT INTO $table (id, name) VALUES (\$1, \$2)',
        <Object?>[i, 'name$i'],
      ));
    }

    for (var i = 1; i <= 50; i++) {
      final name = await db.fetchScalar<String>(
        'SELECT name FROM $table WHERE id = \$1',
        <Object?>[i],
      );
      expect(expectOk(name), 'name$i');
    }
  });

  test('a held statement sees rows written after it was prepared', () async {
    final db = await driver();
    final count = 'SELECT count(*) FROM $table';

    expect(expectOk(await db.fetchScalar<int>(count, const [])), 0);
    expectOk(await db.execute(
      'INSERT INTO $table (id, name) VALUES (\$1, \$2)',
      const <Object?>[1, 'first'],
    ));
    expect(expectOk(await db.fetchScalar<int>(count, const [])), 1);
  });

  test('a schema change under a held statement costs one call, not all of them',
      () async {
    // PostgreSQL refuses a prepared statement whose result type changed —
    // `cached plan must not change result type` — which is what a migration
    // applied while the process runs does to `SELECT *`. The cache parses it
    // again rather than failing from then on.
    final db = await driver();
    final all = 'SELECT * FROM $table';
    expectOk(await db.execute(
      'INSERT INTO $table (id, name) VALUES (\$1, \$2)',
      const <Object?>[1, 'first'],
    ));

    final before = expectOk(await db.fetchAll<int>(
      all,
      const [],
      (row) => row.read<int>('id'),
    ));
    expect(before, <int>[1]);

    expectOk(await db.unsafe.execute(
      'ALTER TABLE $table ADD COLUMN note TEXT',
      const [],
    ));

    final after = expectOk(await db.fetchAll<String?>(
      all,
      const [],
      (row) => row.readNullable<String>('note'),
    ));
    expect(after, <String?>[null]);
  });

  test('more distinct statements than the cache holds still all run', () async {
    final db = await driver();

    // Past the capacity, so the least recently used are evicted and parsed
    // again. Evicting must not break the query that follows it.
    for (var i = 0; i < 100; i++) {
      final value = await db.fetchScalar<int>('SELECT $i', const []);
      expect(expectOk(value), i);
    }

    expect(expectOk(await db.fetchScalar<int>('SELECT 0', const [])), 0);
  });

  test('a transaction runs its own statements, and the pool keeps working',
      () async {
    // A statement prepared inside a transaction would be thrown away with it,
    // so a transaction does not cache. It still has to work.
    final db = await driver();
    final insert = 'INSERT INTO $table (id, name) VALUES (\$1, \$2)';

    expectOk(await db.execute(insert, const <Object?>[1, 'before']));
    expectOk(await db.transaction<Unit>((tx) async {
      final inner = await tx.execute(insert, const <Object?>[2, 'inside']);
      return inner.map((_) => unit);
    }));
    expectOk(await db.execute(insert, const <Object?>[3, 'after']));

    final names = expectOk(await db.fetchAll<String>(
      'SELECT name FROM $table ORDER BY id',
      const [],
      (row) => row.read<String>('name'),
    ));
    expect(names, <String>['before', 'inside', 'after']);
  });

  test('closing releases the statements, and using one afterwards is a value',
      () async {
    final db = connect();
    expectOk(await db.fetchScalar<int>('SELECT 1', const []));
    expectOk(await db.close());

    final afterwards = await db.fetchScalar<int>('SELECT 1', const []);
    expect(afterwards.isErr, isTrue);
  });
}
