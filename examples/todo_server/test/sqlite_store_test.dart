import 'package:test/test.dart';

import 'support/server.dart';

/// The same API, over real SQL. Running the flow twice — once against a map,
/// once against SQLite — is what catches the things a map cannot: an id that
/// comes back as an integer, a boolean that is stored as 0 and 1, a filter
/// that has to become a `WHERE` clause rather than a Dart `if`.

void main() {
  late ExampleServer server;

  setUp(() async => server = await ExampleServer.startWithSqlite());
  tearDown(() => server.stop());

  group('the flow, against SQLite', () {
    test('serves what the schema was seeded with', () async {
      final listed = bodyOf(await server.get('/api/v1/todos'))! as List;

      expect(listed, hasLength(1));
    });

    test('creates and reads back', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));
      final fetched =
          objectOf(await server.get('/api/v1/todos/${created['id']}'));

      expect(fetched, created);
    });

    test('assigns ids the database chose', () async {
      final first = objectOf(await server.post('/api/v1/todos', validBody()));
      final second = objectOf(await server.post('/api/v1/todos', validBody()));

      expect(second['id']! as int, greaterThan(first['id']! as int));
    });

    test('round-trips the done flag through an integer column', () async {
      // SQLite has no boolean; `done` is 0 or 1 on disk and has to come back
      // as JSON `true`.
      final created = objectOf(
        await server.post('/api/v1/todos', validBody(done: true)),
      );

      expect(created['done'], isTrue);
      expect(
        objectOf(await server.get('/api/v1/todos/${created['id']}'))['done'],
        isTrue,
      );
    });

    test('filters by done in SQL, not in Dart', () async {
      final open = objectOf(await server.post('/api/v1/todos', validBody()));
      await server.patch('/api/v1/todos/${open['id']}?done=true');

      final done = bodyOf(await server.get('/api/v1/todos?done=true'))! as List;
      final todo =
          bodyOf(await server.get('/api/v1/todos?done=false'))! as List;

      expect(done, hasLength(1));
      expect(todo, hasLength(1));
    });

    test('scopes the list to the caller with a WHERE clause', () async {
      await server.post(
        '/api/v1/todos',
        validBody(assignTo: other),
        token: adminToken,
      );

      final mine = bodyOf(await server.get('/api/v1/todos'))! as List;
      final all =
          bodyOf(await server.get('/api/v1/todos', token: adminToken))! as List;

      expect(mine, hasLength(1));
      expect(all, hasLength(2));
    });

    test('deletes a row and stops serving it', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));
      final deleted = await server.delete('/api/v1/todos/${created['id']}');

      expect(deleted.statusCode, 204);
      expect(
        (await server.get('/api/v1/todos/${created['id']}')).statusCode,
        404,
      );
    });

    test('answers 400 for an id that is not a number at all', () async {
      // `path<int>` coerces before anything runs, so `abc` is a malformed
      // request rather than a lookup that found nothing — and it never reaches
      // SQLite as a parameter.
      expect((await server.get('/api/v1/todos/abc')).statusCode, 400);
    });

    test('answers 404 for a number no row has', () async {
      expect((await server.get('/api/v1/todos/999999')).statusCode, 404);
    });

    test('reports nothing to the error sink during normal use', () async {
      await server.get('/api/v1/todos');
      await server.get('/api/v1/todos/abc');
      await server.post('/api/v1/todos', validBody());

      expect(server.errors, isEmpty);
    });

    test('keeps validation in front of the database', () async {
      final response = await server.post('/api/v1/todos', validBody(title: ''));

      expect(response.statusCode, 422);
      expect(bodyOf(await server.get('/api/v1/todos'))! as List, hasLength(1));
    });
  });
}
