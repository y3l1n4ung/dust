import 'dart:io';

import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:sqlite3/sqlite3.dart' show Database;
import 'package:test/test.dart';

import 'support/server.dart';

/// The schema comes from `migrations/`, named on `@SqlxDatabase`. Opening the
/// database applies whatever is unapplied, in name order, and records it.
///
/// Two rules follow, and they are the whole discipline:
///
/// 1. **Never edit a migration that has shipped.** It has already run
///    somewhere, so changing it means the schema under test is not the schema
///    anyone is running. Add the next numbered file instead.
/// 2. **Zero-pad the prefixes.** Names sort as text, so `0010` lands after
///    `0009` and `10` does not.

/// The native database underneath, for assertions the DAO deliberately cannot
/// make — reading `sqlite_master`, or writing a row the schema should refuse.
Database _raw(SqliteTodoStore store) =>
    (store.database.connection as Sqlite3Executor).database;

Iterable<String> _migrationFiles() => Directory('migrations')
    .listSync()
    .whereType<File>()
    .map((file) => file.uri.pathSegments.last)
    .where((name) => name.endsWith('.sql'));

void main() {
  group('the folder', () {
    test('holds the files the database is built from', () {
      expect(_migrationFiles().toList()..sort(), [
        '0001_create_todos.sql',
        '0002_index_todos_by_owner.sql',
      ]);
    });

    test('numbers every file, because the name is the schedule', () {
      expect(_migrationFiles(), everyElement(matches(RegExp(r'^\d{4}_'))));
    });
  });

  group('applying them', () {
    late Directory directory;
    late String path;

    setUp(() async {
      directory = await Directory.systemTemp.createTemp('todo_db');
      path = '${directory.path}/todos.db';
    });

    tearDown(() => directory.delete(recursive: true));

    test('builds a schema the API can serve', () async {
      final store = SqliteTodoStore.open(path);
      addTearDown(store.close);

      final created = await store.add(
        const CreateTodo(title: 'first', assignTo: owner),
      );

      expect(created.id, 1);
      expect(await store.all(), hasLength(1));
    });

    test('does not re-run on a database that already has them', () async {
      final first = SqliteTodoStore.open(path);
      await first.add(const CreateTodo(title: 'survives', assignTo: owner));
      await first.close();

      // Re-opening applies only what is unapplied. Re-running `0001` would
      // fail on the existing table, and `0002` on the existing index.
      final second = SqliteTodoStore.open(path);
      addTearDown(second.close);

      final rows = await second.all();
      expect(rows, hasLength(1));
      expect(rows.single.title, 'survives');
    });

    test('keeps the rows across a reopen', () async {
      final first = SqliteTodoStore.open(path);
      for (var index = 0; index < 5; index++) {
        await first.add(CreateTodo(title: 'row $index', assignTo: owner));
      }
      await first.close();

      final second = SqliteTodoStore.open(path);
      addTearDown(second.close);

      expect(await second.all(), hasLength(5));
    });

    test('enforces the CHECK the first migration declares', () async {
      final store = SqliteTodoStore.open(path);
      addTearDown(store.close);

      // `done` is an integer column constrained to 0 or 1, so a stray 7 is
      // refused by SQLite rather than read back as a nonsense boolean.
      expect(
        () => _raw(store).execute(
          'INSERT INTO todos (title, owner, done) VALUES (?, ?, ?)',
          ['bad', owner, 7],
        ),
        throwsA(anything),
      );
    });

    test('creates the index the second migration declares', () async {
      final store = SqliteTodoStore.open(path);
      addTearDown(store.close);

      final indexes = _raw(store)
          .select("SELECT name FROM sqlite_master WHERE type = 'index'")
          .map((row) => row['name'] as String);

      expect(indexes, contains('todos_owner'));
    });

    test('records what it ran, so the ledger survives a restart', () async {
      final store = SqliteTodoStore.open(path);
      addTearDown(store.close);

      final tables = _raw(store)
          .select("SELECT name FROM sqlite_master WHERE type = 'table'")
          .map((row) => row['name'] as String)
          .toList();

      // Something has to remember which migrations already ran, or the second
      // open would try `0001` again.
      expect(tables, contains('todos'));
      expect(tables.length, greaterThan(1));
    });
  });

  group('serving from a file', () {
    test('answers the same as the in-memory store', () async {
      final directory = await Directory.systemTemp.createTemp('todo_db');
      addTearDown(() => directory.delete(recursive: true));

      final server = await ExampleServer.start(
        store: SqliteTodoStore.open('${directory.path}/todos.db'),
      );
      addTearDown(server.stop);

      final created = objectOf(await server.post('/api/v1/todos', validBody()));

      expect(created['owner'], owner);
      expect(
        (await server.get('/api/v1/todos/${created['id']}')).statusCode,
        200,
      );
    });
  });
}
