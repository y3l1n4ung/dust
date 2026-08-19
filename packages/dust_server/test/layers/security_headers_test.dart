import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// None of these headers make a server correct; each turns a class of mistake
/// into a smaller one. What matters is that the defaults are the safe ones and
/// that every one of them can be turned off, because a default nobody can
/// change is a default that gets forked.

void main() {
  Future<Response> send(SecurityHeaders layer, {Response? from}) {
    final handler = layer.toMiddleware()(
      (request) async => from ?? Response.ok('body'),
    );
    return Future.sync(() => handler(request('GET', '/')));
  }

  group('the defaults', () {
    test('stop a browser guessing a type you did not send', () async {
      final response = await send(const SecurityHeaders());

      expect(response.headers['x-content-type-options'], 'nosniff');
    });

    test('stop the page being framed', () async {
      final response = await send(const SecurityHeaders());

      expect(response.headers['x-frame-options'], 'DENY');
    });

    test('limit what the referrer leaks off-site', () async {
      final response = await send(const SecurityHeaders());

      expect(
        response.headers['referrer-policy'],
        'strict-origin-when-cross-origin',
      );
    });

    test('send no CSP, because a wrong one breaks a working page', () async {
      final response = await send(const SecurityHeaders());

      expect(response.headers, isNot(contains('content-security-policy')));
    });

    test('send no HSTS, which over plain HTTP does nothing', () async {
      final response = await send(const SecurityHeaders());

      expect(response.headers, isNot(contains('strict-transport-security')));
    });
  });

  group('overriding', () {
    test('takes a frame policy of its own', () async {
      final response =
          await send(const SecurityHeaders(frameOptions: 'SAMEORIGIN'));

      expect(response.headers['x-frame-options'], 'SAMEORIGIN');
    });

    test('adds a CSP when the application has one', () async {
      final response = await send(
        const SecurityHeaders(contentSecurityPolicy: "default-src 'self'"),
      );

      expect(response.headers['content-security-policy'], "default-src 'self'");
    });

    test('adds HSTS when the deployment warrants it', () async {
      final response = await send(
        const SecurityHeaders(strictTransportSecurity: 'max-age=63072000'),
      );

      expect(
        response.headers['strict-transport-security'],
        'max-age=63072000',
      );
    });

    test('drops a header set to null', () async {
      final response = await send(const SecurityHeaders(frameOptions: null));

      expect(response.headers, isNot(contains('x-frame-options')));
    });

    test('can drop every one of them', () async {
      final response = await send(
        const SecurityHeaders(
          contentTypeOptions: null,
          frameOptions: null,
          referrerPolicy: null,
        ),
      );

      for (final header in [
        'x-content-type-options',
        'x-frame-options',
        'referrer-policy',
      ]) {
        expect(response.headers, isNot(contains(header)));
      }
    });
  });

  group('what it leaves alone', () {
    test('the body', () async {
      final response = await send(const SecurityHeaders());

      expect(await response.readAsString(), 'body');
    });

    test('the status, including a failure', () async {
      final response = await send(
        const SecurityHeaders(),
        from: const Rejection.notFound('gone').intoResponse(),
      );

      expect(response.statusCode, 404);
      expect(response.headers['x-content-type-options'], 'nosniff');
    });
  });

  group('inside a router', () {
    test('covers a 404 as well as a route', () async {
      final app = Router()
        ..layer(const SecurityHeaders())
        ..route('/todos', get((request) async => {'ok': true}));

      final missed = await app.handler(request('GET', '/nothing'));

      expect(missed.statusCode, 404);
      expect(missed.headers['x-frame-options'], 'DENY');
    });
  });
}
