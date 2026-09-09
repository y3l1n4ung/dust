import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

/// Statement reuse, which is most of what a small query costs.
///
/// The cache is private, so these assert on what it is for: the same SQL run
/// many times keeps working, a transaction shares the pool's statements rather
/// than compiling its own, and a closed database leaves none behind.
void main() {
  late Sqlite3Driver db;

  setUp(() {
    db = Sqlite3Driver.connect(
      const SqliteConnectOptions.memory(),
      migrations: const <String, String>{
        '0001.sql': '''
CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
''',
      },
    );
  });

  tearDown(() async {
    await db.close();
  });

  test('the same statement run many times keeps its parameters straight',
      () async {
    for (var i = 1; i <= 50; i++) {
      final inserted = await db.execute(
        r'INSERT INTO items (id, name) VALUES ($1, $2)',
        <Object?>[i, 'name$i'],
      );
      expect(inserted.isOk, isTrue);
    }

    for (var i = 1; i <= 50; i++) {
      final name = await db.fetchScalar<String>(
        r'SELECT name FROM items WHERE id = $1',
        <Object?>[i],
      );
      expect(name.unwrapOrElse((error) => throw error), 'name$i');
    }
  });

  test('a held statement still sees rows written after it was prepared',
      () async {
    const select = 'SELECT count(*) FROM items';
    expect((await db.fetchScalar<int>(select, const [])).unwrapOr(-1), 0);

    await db.execute(
      r'INSERT INTO items (name) VALUES ($1)',
      const <Object?>['first'],
    );

    expect((await db.fetchScalar<int>(select, const [])).unwrapOr(-1), 1);
  });

  test('a statement survives the schema changing under it', () async {
    // SQLite recompiles a prepared statement when the schema moves, and the
    // cache has to be no worse than that.
    const select = 'SELECT count(*) FROM items';
    expect((await db.fetchScalar<int>(select, const [])).unwrapOr(-1), 0);

    await Sqlite3UnsafeSql(db)
        .execute('ALTER TABLE items ADD COLUMN note TEXT', const []);

    expect((await db.fetchScalar<int>(select, const [])).unwrapOr(-1), 0);
  });

  test('a held statement follows the table being rebuilt under it', () async {
    // Harsher than adding a column: the table is dropped and recreated with a
    // different shape while a statement for it is held. SQLite recompiles on
    // SQLITE_SCHEMA, and the cache must not defeat that by handing back a
    // statement compiled against the old table.
    const count = 'SELECT count(*) FROM items';
    expect((await db.fetchScalar<int>(count, const [])).unwrapOr(-1), 0);

    final unsafe = Sqlite3UnsafeSql(db);
    expect((await unsafe.execute('DROP TABLE items', const [])).isOk, isTrue);
    expect(
      (await unsafe.execute(
        'CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL, '
        'note TEXT)',
        const [],
      ))
          .isOk,
      isTrue,
    );
    await db.execute(
      r'INSERT INTO items (name, note) VALUES ($1, $2)',
      const <Object?>['rebuilt', 'and noted'],
    );

    expect((await db.fetchScalar<int>(count, const [])).unwrapOr(-1), 1);

    final note = await db.fetchScalar<String>(
      'SELECT note FROM items',
      const [],
    );
    expect(note.unwrapOrElse((error) => throw error), 'and noted');
  });

  test('a statement held for a table that goes away reports it as a value',
      () async {
    const count = 'SELECT count(*) FROM items';
    expect((await db.fetchScalar<int>(count, const [])).unwrapOr(-1), 0);

    expect(
      (await Sqlite3UnsafeSql(db).execute('DROP TABLE items', const [])).isOk,
      isTrue,
    );

    final gone = await db.fetchScalar<int>(count, const []);
    expect(gone.isErr, isTrue);
  });

  test('a transaction runs the pool statements, and both stay usable',
      () async {
    const insert = r'INSERT INTO items (name) VALUES ($1)';
    await db.execute(insert, const <Object?>['before']);

    final committed = await db.transaction<Unit>((tx) async {
      final inner = await tx.execute(insert, const <Object?>['inside']);
      return inner.map((_) => unit);
    });
    expect(committed.isOk, isTrue);

    await db.execute(insert, const <Object?>['after']);

    final names = await db.fetchAll<String>(
      'SELECT name FROM items ORDER BY id',
      const [],
      (row) => row.read<String>('name'),
    );
    expect(names.unwrapOrElse((error) => throw error),
        <String>['before', 'inside', 'after']);
  });

  test('more distinct statements than the cache holds still all run', () async {
    // Past the capacity, so the least recently used are evicted and prepared
    // again. Evicting a statement must not break the query that follows it.
    for (var i = 0; i < 200; i++) {
      final value = await db.fetchScalar<int>('SELECT $i', const []);
      expect(value.unwrapOrElse((error) => throw error), i);
    }

    final again = await db.fetchScalar<int>('SELECT 0', const []);
    expect(again.unwrapOrElse((error) => throw error), 0);
  });

  test('many transactions in a row each commit their own work', () async {
    // BEGIN, COMMIT and ROLLBACK are held like any other statement, so a
    // transaction per request reuses all three. Reusing one wrongly would show
    // up as work landing in the wrong transaction, or not landing at all.
    for (var i = 1; i <= 30; i++) {
      final committed = await db.transaction<Unit>((tx) async {
        final inserted = await tx.execute(
          r'INSERT INTO items (id, name) VALUES ($1, $2)',
          <Object?>[i, 'name$i'],
        );
        return inserted.map((_) => unit);
      });
      expect(committed.isOk, isTrue);

      // Every odd row is written and then rolled back, so it must not survive.
      if (i.isOdd) {
        await db.transaction<Unit>((tx) async {
          await tx.execute(
            r'INSERT INTO items (id, name) VALUES ($1, $2)',
            <Object?>[1000 + i, 'rolled back'],
          );
          return Err<Unit, SqlxError>(
            SqlxError.query('undo', operation: 'test'),
          );
        });
      }
    }

    final kept = await db.fetchScalar<int>(
      'SELECT count(*) FROM items',
      const [],
    );
    expect(kept.unwrapOrElse((error) => throw error), 30);
  });

  test('nested savepoints repeat without colliding', () async {
    // Savepoint names carry a counter and are deliberately not held, so this
    // is the case that would break if they ever were.
    for (var i = 1; i <= 10; i++) {
      await db.transaction<Unit>((tx) async {
        await tx.execute(
          r'INSERT INTO items (id, name) VALUES ($1, $2)',
          <Object?>[i, 'outer$i'],
        );
        await tx.transaction<Unit>((nested) async {
          await nested.execute(
            r'INSERT INTO items (id, name) VALUES ($1, $2)',
            <Object?>[1000 + i, 'inner$i'],
          );
          return Err<Unit, SqlxError>(
            SqlxError.query('undo', operation: 'test'),
          );
        });
        return const Ok<Unit, SqlxError>(unit);
      });
    }

    final names = await db.fetchAll<String>(
      'SELECT name FROM items ORDER BY id',
      const [],
      (row) => row.read<String>('name'),
    );
    expect(
      names.unwrapOrElse((error) => throw error),
      <String>[for (var i = 1; i <= 10; i++) 'outer$i'],
    );
  });

  test('closing releases the statements, and using one afterwards is a value',
      () async {
    await db.fetchScalar<int>('SELECT count(*) FROM items', const []);
    expect((await db.close()).isOk, isTrue);

    final afterwards = await db.fetchScalar<int>(
      'SELECT count(*) FROM items',
      const [],
    );
    expect(afterwards.isErr, isTrue);
  });
}
