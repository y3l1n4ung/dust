import 'package:test/test.dart';

import 'support/server.dart';

/// What a client is told when a body is well-formed JSON that breaks a
/// `@Validate` constraint. The status has to be 422 rather than 400, and the
/// body has to name the field and repeat the message from the annotation:
/// a client that cannot tell which input to fix is being told nothing.

void main() {
  late ExampleServer server;

  setUp(() async => server = await ExampleServer.start());
  tearDown(() => server.stop());

  group('a failed constraint', () {
    test('answers 422 rather than 400', () async {
      final response = await server.post('/api/v1/todos', validBody(title: ''));

      expect(response.statusCode, 422);
    });

    test('names the field that failed', () async {
      final response = await server.post('/api/v1/todos', validBody(title: ''));

      expect(fieldsOf(response), contains('title'));
    });

    test('repeats the message from the annotation', () async {
      final response = await server.post('/api/v1/todos', validBody(title: ''));

      expect(fieldsOf(response)['title'], ['must be 1 to 200 characters']);
    });

    test('reports every failed field, not just the first', () async {
      final response = await server.post(
        '/api/v1/todos',
        validBody(title: '', assignTo: 'not-an-email'),
      );

      expect(fieldsOf(response).keys, containsAll(['title', 'assignTo']));
      expect(fieldsOf(response)['assignTo'], ['must be an email address']);
    });

    test('separates the constraint failure from a shape failure', () async {
      final response = await server.post('/api/v1/todos', validBody(title: ''));

      expect(objectOf(response)['error'], 'Validation failed');
    });

    test('enforces the upper bound as well as the lower one', () async {
      final response = await server.post(
        '/api/v1/todos',
        validBody(title: 'x' * 201),
      );

      expect(fieldsOf(response)['title'], ['must be 1 to 200 characters']);
    });

    test('accepts a title at each end of the allowed range', () async {
      final shortest =
          await server.post('/api/v1/todos', validBody(title: 'a'));
      final longest =
          await server.post('/api/v1/todos', validBody(title: 'x' * 200));

      expect([shortest.statusCode, longest.statusCode], [201, 201]);
    });

    test('stores nothing when validation fails', () async {
      await server.post('/api/v1/todos', validBody(title: ''));
      final listed = bodyOf(await server.get('/api/v1/todos'))! as List;

      expect(listed, hasLength(1));
    });

    test('is not reported as a server error', () async {
      await server.post('/api/v1/todos', validBody(title: ''));

      expect(server.errors, isEmpty);
    });
  });

  group('a body of the wrong shape', () {
    test('answers 422 with no fields, since none can be blamed', () async {
      final response = await server.post(
        '/api/v1/todos',
        <String, Object?>{'title': 7, 'assignTo': owner},
      );

      expect(response.statusCode, 422);
      expect(objectOf(response), isNot(contains('fields')));
    });

    test('answers 400 for malformed JSON', () async {
      final response = await server.post('/api/v1/todos', '{not json');

      expect(response.statusCode, 400);
    });

    test('answers 415 for the wrong content type', () async {
      final response = await server.post(
        '/api/v1/todos',
        validBody(),
        contentType: 'text/plain',
      );

      expect(response.statusCode, 415);
    });
  });
}
