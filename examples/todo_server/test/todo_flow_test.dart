import 'package:test/test.dart';

import 'support/server.dart';

/// The CRUD path a client actually walks, end to end over a socket.

void main() {
  late ExampleServer server;

  setUp(() async => server = await ExampleServer.start());
  tearDown(() => server.stop());

  group('the todo API', () {
    test('serves what the repository was seeded with', () async {
      final listed = bodyOf(await server.get('/api/v1/todos'))! as List;

      expect(listed, hasLength(1));
    });

    test('answers 201 on create', () async {
      final response = await server.post('/api/v1/todos', validBody());

      expect(response.statusCode, 201);
    });

    test('adds the created todo to the list', () async {
      await server.post('/api/v1/todos', validBody());
      final listed = bodyOf(await server.get('/api/v1/todos'))! as List;

      expect(listed, hasLength(2));
    });

    test('reads one back by the id it assigned', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));
      final fetched = await server.get('/api/v1/todos/${created['id']}');

      expect(fetched.statusCode, 200);
    });

    test('marks one done and reports the new state', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));
      final patched = objectOf(
        await server.patch('/api/v1/todos/${created['id']}?done=true'),
      );

      expect(patched['done'], isTrue);
    });

    test('filters the list by the done flag', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));
      await server.patch('/api/v1/todos/${created['id']}?done=true');

      final done = bodyOf(await server.get('/api/v1/todos?done=true'))! as List;
      final open =
          bodyOf(await server.get('/api/v1/todos?done=false'))! as List;

      expect([done.length, open.length], [1, 1]);
    });

    test('answers 204 with no body on delete', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));
      final deleted = await server.delete('/api/v1/todos/${created['id']}');

      expect(deleted.statusCode, 204);
      expect(deleted.body, isEmpty);
    });

    test('stops serving a deleted todo', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));
      await server.delete('/api/v1/todos/${created['id']}');

      final fetched = await server.get('/api/v1/todos/${created['id']}');

      expect(fetched.statusCode, 404);
    });

    test('serves the health route outside the versioned prefix', () async {
      final response = await server.get('/health', token: null);

      expect(objectOf(response), {'ok': true});
    });

    test('records one access line per request', () async {
      await server.get('/health', token: null);
      await server.get('/api/v1/todos');

      expect(server.records, hasLength(2));
    });

    test('gives every request an id', () async {
      final response = await server.get('/health', token: null);

      expect(response.headers['x-request-id'], isNotNull);
    });
  });
}
