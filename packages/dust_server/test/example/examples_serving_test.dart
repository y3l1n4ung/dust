import 'dart:convert';
import 'dart:io';
import 'package:test/test.dart';
import '../../example/several_isolates.dart' as several_isolates;
import '../../example/isolate_failure.dart' as isolate_failure;
import '../../example/router_as_handler.dart' as router_as_handler;
import '../../example/graceful_shutdown.dart' as graceful_shutdown;
import '../../example/tls.dart' as tls;
import '../../example/tracing.dart' as tracing;
import 'serve.dart';
import 'support.dart';

/// Running the server: isolates, shutdown, TLS and observability.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
void main() {
  group('graceful_shutdown', () {
    test('a request already accepted finishes after close begins', () async {
      // The whole point. A process that exits on the signal drops these, and
      // they are the slow ones — the ones most likely to be mid-write.
      final app = await ExampleApp.start(graceful_shutdown.buildApp());

      final slow = app.get('/slow');
      await Future<void>.delayed(const Duration(milliseconds: 100));
      final settled = await app.stop();

      expect(settled, isTrue);
      expect((await slow).statusCode, 200);
    });

    test('close reports whether everything finished', () async {
      // The return value is the only way to learn that requests were abandoned,
      // and it is the thing most code throws away.
      final app = await ExampleApp.start(graceful_shutdown.buildApp());

      await app.get('/quick');

      expect(await app.stop(), isTrue);
    });

    test('nothing new is accepted once close has begun', () async {
      final app = await ExampleApp.start(graceful_shutdown.buildApp());
      final origin = app.origin;

      await app.stop();

      await expectLater(
        HttpClient().getUrl(Uri.parse('$origin/quick')).then((r) => r.close()),
        throwsA(isA<SocketException>()),
      );
    });
  });

  group('tracing', () {
    test('the span is named after the route, not the URL', () async {
      // /orders/41 and /orders/42 must share a name, or a dashboard has one
      // series per order and nothing can group it.
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/orders/41');
      await app.get('/orders/42');

      expect(
        spans.spans.map((span) => span.name),
        ['GET /orders/{id}', 'GET /orders/{id}'],
      );
    });

    test('a 404 is traced too', () async {
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/nothing');

      expect(spans.spans, hasLength(1));
      expect(spans.spans.single.attributes['http.response.status_code'], 404);
    });

    test('an incoming traceparent is continued, not replaced', () async {
      const traceId = '4bf92f3577b34da6a3ce929d0e0e4736';
      const parentId = '00f067aa0ba902b7';
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get(
        '/orders/41',
        headers: {'traceparent': '00-$traceId-$parentId-01'},
      );

      expect(spans.spans.single.context.traceId, traceId);
      expect(spans.spans.single.parentSpanId, parentId);
    });

    test('an absent traceparent starts a trace', () async {
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/orders/41');

      expect(spans.spans.single.context.traceId, hasLength(32));
      expect(spans.spans.single.parentSpanId, isNull);
    });

    test('attributes set inside the handler reach the span', () async {
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/orders/41');

      expect(spans.spans.single.attributes['order.id'], 41);
      expect(spans.spans.single.attributes['cache.hit'], false);
    });

    test('nameSpan overrides the route name', () async {
      final spans = CollectingExporter();
      final app = await example(tracing.buildApp(exporter: spans));

      await app.get('/legacy/rebuild');

      expect(spans.spans.single.name, 'legacy.rebuild');
    });
  });

  group('isolate_failure', () {
    // The isolate machinery itself is covered by test/serving/isolates_test.dart.
    // What matters here is that the application each isolate builds is real.
    test('builds an application that reports which process answered', () async {
      final app = await example(isolate_failure.buildApp());

      expect((await app.get('/')).body, 'ok');
      expect(jsonDecode((await app.get('/health')).body), {'pid': pid});
    });
  });

  group('router_as_handler', () {
    test('serves the inner router through a shelf pipeline', () async {
      final app = await example(router_as_handler.buildApp());

      final root = await app.get('/');
      expect(root.body, 'from the inner router');
      expect(root.headers['x-served-through'], 'shelf-pipeline');

      final who = await app.get('/who');
      expect(jsonDecode(who.body), {'router': 'inner'});
    });
  });

  group('several_isolates', () {
    test('the factory builds a working application on its own', () async {
      // The cluster itself is covered by the runtime's serving tests. What this
      // example owns is that its factory is a valid top-level one.
      final app = await example(several_isolates.buildApp());

      final response = await app.get('/whoami');

      expect(response.statusCode, 200);
      expect(app.object(response)['seen'], 1);
    });

    test('state is per-application, which is per-isolate in a cluster',
        () async {
      // Two applications from one factory share nothing. In a cluster that is
      // exactly what each isolate gets, and why an in-memory counter counts a
      // fraction of the traffic.
      final first = await example(several_isolates.buildApp());
      final second = await example(several_isolates.buildApp());

      await first.get('/whoami');
      await first.get('/whoami');

      expect(app0(await first.get('/whoami')), 3);
      expect(app0(await second.get('/whoami')), 1);
    });
  });

  group('tls', () {
    test('the application serves plainly, so TLS is a deployment choice',
        () async {
      final app = await example(tls.buildApp());

      expect(app.object(await app.get('/health')), {'status': 'ok'});
    });

    test('no HSTS without a real certificate', () async {
      // Sent from a host whose certificate later lapses, it locks users out.
      final app = await example(tls.buildApp());

      expect(
        (await app.get('/health')).headers['strict-transport-security'],
        isNull,
      );
    });

    test('the redirect application sends everything to https with 308',
        () async {
      // 308, so a POST is not silently turned into a GET and stripped of its
      // body.
      final app = await example(tls.buildRedirectApp(port: 8443));

      final response = await app.raw('GET', '/health');

      expect(response.statusCode, 308);
      expect(response.headers['location'], startsWith('https://'));
      expect(response.headers['location'], contains(':8443/health'));
    });

    test('port 443 is left out of the redirect target', () async {
      final app = await example(tls.buildRedirectApp(port: 443));

      final location = (await app.raw('GET', '/health')).headers['location']!;

      expect(location, isNot(contains(':443')));
    });
  });
}
