import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// A layer that owns something and records when it was released.
final class _Owning implements DisposableLayer {
  _Owning(this.name, this.closed);

  final String name;
  final List<String> closed;

  @override
  Middleware toMiddleware() => (inner) => inner;

  @override
  Future<void> dispose() async => closed.add(name);
}

final class _Throwing implements DisposableLayer {
  _Throwing(this.after);

  final List<String> after;

  @override
  Middleware toMiddleware() => (inner) => inner;

  @override
  Future<void> dispose() async => throw StateError('bad teardown');
}

void main() {
  group('DisposableLayer', () {
    test('is disposed when the server closes', () async {
      final closed = <String>[];
      final app = Router()
        ..layer(_Owning('outer', closed))
        ..route('/', get((request) => 'ok'));

      final server = await serve(app, InternetAddress.loopbackIPv4, 0);
      expect(closed, isEmpty, reason: 'not until shutdown');

      await server.close(drain: const Duration(seconds: 1));
      expect(closed, ['outer']);
    });

    test('is found through nest and routeLayer', () async {
      final closed = <String>[];
      final inner = Router()
        ..routeLayer(_Owning('nested-route', closed))
        ..route('/deep', get((request) => 'ok'));
      final app = Router()
        ..layer(_Owning('root', closed))
        ..nest('/api', inner);

      final server = await serve(app, InternetAddress.loopbackIPv4, 0);
      await server.close(drain: const Duration(seconds: 1));

      expect(closed, containsAll(['root', 'nested-route']));
    });

    test('a plain Layer is left alone', () async {
      final app = Router()
        ..layer(RequestId())
        ..route('/', get((request) => 'ok'));

      final server = await serve(app, InternetAddress.loopbackIPv4, 0);
      await expectLater(
        server.close(drain: const Duration(seconds: 1)),
        completion(isTrue),
      );
    });

    test('one that throws does not stop the others', () async {
      final closed = <String>[];
      final app = Router()
        ..layer(_Throwing(closed))
        ..layer(_Owning('after-the-bad-one', closed))
        ..route('/', get((request) => 'ok'));

      final server = await serve(app, InternetAddress.loopbackIPv4, 0);
      await server.close(drain: const Duration(seconds: 1));

      expect(closed, ['after-the-bad-one']);
    });

    test('is disposed once, not again on a second close', () async {
      final closed = <String>[];
      final app = Router()
        ..layer(_Owning('once', closed))
        ..route('/', get((request) => 'ok'));

      final server = await serve(app, InternetAddress.loopbackIPv4, 0);
      await server.close(drain: const Duration(seconds: 1));
      await server.close(drain: const Duration(seconds: 1));

      expect(closed, ['once']);
    });
  });

  group('Router as a Service', () {
    test('answers a request directly', () async {
      final app = Router()..route('/', get((request) => 'hello'));

      final response = await app(Request('GET', Uri.parse('http://x/')));
      expect(await response.readAsString(), 'hello');
    });

    test('tears off where a Handler is wanted', () async {
      final app = Router()..route('/', get((request) => 'hello'));

      final handler = app.call;
      final response = await handler(Request('GET', Uri.parse('http://x/')));
      expect(response.statusCode, 200);
    });

    test('is a Service', () {
      expect(Router(), isA<Service>());
    });
  });
}
