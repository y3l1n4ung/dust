import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../../support.dart';

/// `TRACE` and `CONNECT` are rarely wanted, and `any` already covered them.
/// Having named builders means a router can answer one deliberately rather
/// than by accident, and can still refuse it with a 405 when it has not.

void main() {
  group('TRACE', () {
    test('is served when a route claims it', () async {
      final app = Router()
        ..route('/echo', trace((request) async => {'traced': true}));

      final response = await app.handler(request('TRACE', '/echo'));

      expect(response.statusCode, 200);
    });

    test('is a 405 on a path that serves only GET', () async {
      // The safe default: nothing echoes a request back unless asked to.
      final app = Router()
        ..route('/echo', get((request) async => {'ok': true}));

      final response = await app.handler(request('TRACE', '/echo'));

      expect(response.statusCode, 405);
      expect(response.headers['allow'], 'GET, HEAD');
    });

    test('chains after another verb on the same path', () async {
      final app = Router()
        ..route(
          '/echo',
          get((request) async => {'get': true})
              .trace((request) async => {'trace': true}),
        );

      expect((await app.handler(request('GET', '/echo'))).statusCode, 200);
      expect((await app.handler(request('TRACE', '/echo'))).statusCode, 200);
    });

    test('appears in Allow once it is registered', () async {
      final app = Router()
        ..route(
          '/echo',
          get((request) async => {'ok': true})
              .trace((request) async => {'ok': true}),
        );

      final response = await app.handler(request('DELETE', '/echo'));

      expect(response.headers['allow'], 'GET, HEAD, TRACE');
    });
  });

  group('CONNECT', () {
    test('is served when a route claims it', () async {
      final app = Router()
        ..route('/tunnel', connect((request) async => {'open': true}));

      expect(
          (await app.handler(request('CONNECT', '/tunnel'))).statusCode, 200);
    });

    test('is a 405 otherwise', () async {
      final app = Router()
        ..route('/tunnel', get((request) async => {'ok': true}));

      expect(
          (await app.handler(request('CONNECT', '/tunnel'))).statusCode, 405);
    });

    test('chains like any other verb', () async {
      final app = Router()
        ..route(
          '/tunnel',
          post((request) async => {'post': true})
              .connect((request) async => {'connect': true}),
        );

      expect((await app.handler(request('POST', '/tunnel'))).statusCode, 200);
      expect(
          (await app.handler(request('CONNECT', '/tunnel'))).statusCode, 200);
    });
  });

  group('either one', () {
    test('refuses to be registered twice on a path', () {
      expect(
        () => trace((request) async => 1).trace((request) async => 2),
        throwsArgumentError,
      );
    });

    test('takes a success status like the rest', () async {
      final app = Router()
        ..route('/t', trace((request) async => {'ok': true}, status: 202));

      expect((await app.handler(request('TRACE', '/t'))).statusCode, 202);
    });
  });
}
