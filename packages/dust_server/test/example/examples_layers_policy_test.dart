import 'dart:io';
import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';
import '../../example/disposable_layers.dart' as disposable_layers;
import '../../example/normalize_path.dart' as normalize_path;
import '../../example/request_timeout.dart' as request_timeout;
import '../../example/route_layer.dart' as route_layer;
import '../../example/security_headers.dart' as security_headers;
import 'serve.dart';

/// Layers wrapped around a handler, and the order they run in.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
/// Path normalization, security headers, per-route layers and timeouts.
void main() {
  group('normalize_path', () {
    test('a trailing slash is rewritten, and the client sees one response',
        () async {
      final app = await example(normalize_path.buildApp());

      final bare = await app.get('/notes');
      final slashed = await app.get('/notes/');

      expect(slashed.statusCode, 200);
      expect(slashed.body, bare.body);
    });

    test('a nested route is covered when the layer sits above the nest',
        () async {
      final app = await example(normalize_path.buildApp());

      expect((await app.get('/api/notes/')).statusCode, 200);
    });

    test('a layer on a nested router covers that prefix, 404s included',
        () async {
      final app = await example(normalize_path.buildScopedApp());

      expect((await app.get('/api/notes')).statusCode, 200);
      expect((await app.get('/api/notes/')).statusCode, 200);
    });

    test('and covers nothing outside it', () async {
      // Scoping matters when one half of an application has URLs you must not
      // rewrite — a webhook whose signature covers the exact path, say.
      final app = await example(normalize_path.buildScopedApp());

      expect((await app.get('/shop/notes')).statusCode, 200);
      expect((await app.get('/shop/notes/')).statusCode, 404);
    });

    test('the root is never touched', () async {
      // Stripping its slash would leave an empty path nothing can match.
      final app = await example(normalize_path.buildApp());

      expect(app.object(await app.get('/')), {'root': true});
    });
  });

  group('security_headers', () {
    test('a page carries the whole set, CSP included', () async {
      final app = await example(security_headers.buildApp());

      final response = await app.get('/app');

      expect(response.headers['x-content-type-options'], 'nosniff');
      expect(response.headers['x-frame-options'], 'DENY');
      expect(response.headers['referrer-policy'], isNotNull);
      expect(response.headers['content-security-policy'], contains("'self'"));
    });

    test('the CSP names its sources and allows no inline script', () async {
      // An allowlist that permits inline scripts permits the injected one too.
      final app = await example(security_headers.buildApp());

      final policy =
          (await app.get('/app')).headers['content-security-policy']!;

      expect(policy, isNot(contains('unsafe-inline')));
      expect(policy, contains("frame-ancestors 'none'"));
    });

    test('the API gets the cheap headers and no CSP', () async {
      // Two nested routers, so each policy covers one half. A merged router has
      // no prefix, and its layer would cover both.
      final app = await example(security_headers.buildApp());

      final response = await app.get('/api/notes');

      expect(response.headers['x-content-type-options'], 'nosniff');
      expect(response.headers['content-security-policy'], isNull);
    });

    test('HSTS is absent, because this example serves plain HTTP', () async {
      final app = await example(security_headers.buildApp());

      expect(
        (await app.get('/app')).headers['strict-transport-security'],
        isNull,
      );
    });
  });

  group('route_layer', () {
    test('a matched route without a credential is 401', () async {
      final app = await example(route_layer.buildApp());

      final response = await app.get('/admin/orders');

      expect(response.statusCode, 401);
      expect(response.headers['www-authenticate'], contains('Bearer'));
    });

    test('the credential gets through', () async {
      final app = await example(route_layer.buildApp());

      final response = await app.get(
        '/admin/orders',
        headers: {'authorization': 'Bearer staff'},
      );

      expect(app.array(response), ['order-1']);
    });

    test('an unmatched path under the prefix is 404, not 401', () async {
      // The whole reason for routeLayer. With a plain layer this answers 401,
      // and a typo in your own route table looks like an auth problem.
      final app = await example(route_layer.buildApp());

      expect((await app.get('/admin/typo')).statusCode, 404);
    });

    test('routes outside the guard are untouched', () async {
      final app = await example(route_layer.buildApp());

      expect((await app.get('/health')).statusCode, 200);
    });

    test('a real credential that is not the staff one is 403', () async {
      final app = await example(route_layer.buildApp());

      final response = await app.get(
        '/admin/orders',
        headers: {'authorization': 'Bearer intern'},
      );

      expect(response.statusCode, 403);
    });
  });

  group('request_timeout', () {
    test('a request inside the budget is untouched', () async {
      final app = await example(request_timeout.buildApp(onTimeout: (_) {}));

      expect(app.object(await app.get('/quick')), {'ok': true});
    });

    test('a request over the budget is a 503', () async {
      final app = await example(request_timeout.buildApp(onTimeout: (_) {}));

      final response = await app.get('/slow');

      expect(response.statusCode, 503);
      expect(app.object(response)['error'], contains('200ms'));
    });

    test('a streamed body is not bounded by the deadline', () async {
      // The trap: the budget covers producing the Response, not sending it. A
      // handler that returns at once with a stream has already met it.
      final app = await example(
        Router()
          ..layer(const RequestTimeout(Duration(milliseconds: 50)))
          ..route(
            '/events',
            get((request) => eventStream(
                  Stream<ServerSentEvent>.periodic(
                    const Duration(milliseconds: 40),
                    (index) => ServerSentEvent(data: '$index'),
                  ).take(4),
                  keepAlive: null,
                )),
          ),
      );

      final response = await app.get('/events');

      // 200 after ~160ms, well past the 50ms budget.
      expect(response.statusCode, 200);
      expect(response.body, contains('data:3'));
    });

    test('onTimeout fires, so a 503 can be counted', () async {
      // A 503 nobody counted is an outage nobody noticed.
      final timedOut = <String>[];
      final app = await example(
        request_timeout.buildApp(
          onTimeout: (request) => timedOut.add(request.url.path),
        ),
      );

      await app.get('/slow');

      expect(timedOut, ['slow']);
    });
  });

  group('disposable_layers', () {
    test('serves, and flushes the counter exactly once on shutdown', () async {
      final flushed = <String>[];
      final app = Router()
        ..layer(disposable_layers.RequestCounter(flushed.add))
        ..route('/', get((request) async => 'counted'));

      final server = await serve(app, InternetAddress.loopbackIPv4, 0);
      final origin = 'http://127.0.0.1:${server.port}';

      expect((await http.get(Uri.parse('$origin/'))).body, 'counted');
      expect((await http.get(Uri.parse('$origin/'))).body, 'counted');
      expect(flushed, isEmpty, reason: 'not until shutdown');

      await server.close(drain: const Duration(seconds: 1));
      expect(flushed, ['flushed 2 requests']);
    });
  });
}
