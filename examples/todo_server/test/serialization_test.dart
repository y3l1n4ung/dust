import 'package:test/test.dart';

import 'support/server.dart';

/// The generated `serialize` and `deserialize` are what the wire format is.
/// These check them where it matters — after the bytes have crossed a socket —
/// rather than by calling the methods directly, which would prove only that
/// the generator ran.

void main() {
  late ExampleServer server;

  setUp(() async => server = await ExampleServer.start());
  tearDown(() => server.stop());

  group('the generated serializer', () {
    test('writes every declared field', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));

      expect(created.keys, containsAll(['id', 'title', 'owner', 'done']));
    });

    test('writes types JSON can carry, not Dart toString output', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));

      expect(created['title'], isA<String>());
      expect(created['done'], isA<bool>());
    });

    test('runs over the elements of a list without a manual map', () async {
      await server.post('/api/v1/todos', validBody(title: 'second'));
      final listed = bodyOf(await server.get('/api/v1/todos'))! as List;

      expect(listed, everyElement(isA<Map<String, Object?>>()));
      expect(listed, hasLength(2));
    });

    test('does not leak the model that owns the fields', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));

      expect(created.keys, isNot(contains('runtimeType')));
    });
  });

  group('the generated deserializer', () {
    test('reads a field the request supplied', () async {
      final created = objectOf(
        await server.post('/api/v1/todos', validBody(title: 'read me back')),
      );

      expect(created['title'], 'read me back');
    });

    test('applies the declared default for an absent field', () async {
      final created = objectOf(
        await server.post(
          '/api/v1/todos',
          <String, Object?>{'title': 'no done key', 'assignTo': owner},
        ),
      );

      expect(created['done'], isFalse);
    });

    test('carries a supplied value through to the response', () async {
      final created = objectOf(
        await server.post('/api/v1/todos', validBody(done: true)),
      );

      expect(created['done'], isTrue);
    });

    test('rejects a missing required field', () async {
      final response = await server.post(
        '/api/v1/todos',
        <String, Object?>{'assignTo': owner},
      );

      expect(response.statusCode, 422);
    });
  });

  group('a round trip', () {
    test('reads back what the create wrote', () async {
      final created = objectOf(
        await server.post('/api/v1/todos', validBody(title: 'round trip')),
      );
      final fetched =
          objectOf(await server.get('/api/v1/todos/${created['id']}'));

      expect(fetched, created);
    });

    test('survives a value that needs JSON escaping', () async {
      const awkward = 'quote " backslash \\ newline \n emoji 🚀';
      final created = objectOf(
        await server.post('/api/v1/todos', validBody(title: awkward)),
      );

      expect(created['title'], awkward);
    });

    test('survives a value that is not ASCII', () async {
      final created = objectOf(
        await server.post('/api/v1/todos', validBody(title: 'မင်္ဂလာပါ')),
      );
      final fetched =
          objectOf(await server.get('/api/v1/todos/${created['id']}'));

      expect(fetched['title'], 'မင်္ဂလာပါ');
    });
  });
}
