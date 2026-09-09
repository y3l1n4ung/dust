import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// What the layer has to get right: continue an incoming trace rather than
/// starting a new one, hand the span back on the response, call a 5xx an error
/// and a 404 not one, and never let the exporter decide whether the request
/// succeeded.

final class _BrokenExporter implements SpanExporter {
  @override
  void export(Span span) => throw StateError('the collector is down');
}

/// Spans created outside the layer, the exporter, and the current span.
void main() {
  late RecordingSpanExporter exporter;

  setUp(() => exporter = RecordingSpanExporter());

  Handler build({
    SpanExporter? into,
    String? serviceName,
    Object? Function(Request request)? handler,
  }) {
    return (Router()
          ..layer(Tracing(into ?? exporter, serviceName: serviceName))
          ..route(
              '/todos/{id}', get(handler ?? (request) async => {'ok': true}))
          ..route('/boom', get((request) async => throw StateError('boom')))
          ..route('/missing',
              get((request) async => const Rejection.notFound('x'))))
        .handler;
  }

  Future<Response> send(
    Handler app,
    String path, {
    Map<String, String> headers = const {},
  }) =>
      Future.sync(() => app(request('GET', path, headers: headers)));

  group('a span on its own', () {
    test('records several attributes at once', () {
      final span = Span(name: 'x', context: TraceContext.start())
        ..setAttributes({'a': 1, 'b': 2});

      expect(span.attributes, containsPair('a', 1));
      expect(span.attributes, containsPair('b', 2));
    });

    test('keeps the first end time when ended twice', () {
      final span = Span(name: 'x', context: TraceContext.start())..end();
      final first = span.endedAt;
      span.end();

      expect(span.endedAt, first);
    });

    test('describes itself with its ids and status', () {
      final span = Span(name: 'db.query', context: TraceContext.start())
        ..end(status: SpanStatus.ok);

      expect(span.toString(), contains('db.query'));
      expect(span.toString(), contains(span.context.traceId));
      expect(span.toString(), contains('SpanStatus.ok'));
    });
  });

  group('the exporter', () {
    test('cannot fail the request it is reporting on', () async {
      final response = await send(build(into: _BrokenExporter()), '/todos/7');

      expect(response.statusCode, 200);
    });
  });

  group('the current span', () {
    test('is reachable from the handler', () async {
      String? seen;
      await send(
        build(handler: (request) {
          seen = CurrentSpan.value?.context.spanId;
          return {'ok': true};
        }),
        '/todos/7',
      );

      expect(seen, exporter.spans.single.context.spanId);
    });

    test('takes attributes a handler records', () async {
      await send(
        build(handler: (request) {
          CurrentSpan.setAttribute('todo.id', '7');
          return {'ok': true};
        }),
        '/todos/7',
      );

      expect(exporter.spans.single.attributes['todo.id'], '7');
    });

    test('is named after the route without being asked', () async {
      // The route is only known after matching, so the layer renames the span
      // from `matchedPathOf` once the handler returns. Nothing in the handler
      // has to remember to do it.
      await send(build(), '/todos/7');

      expect(exporter.spans.single.name, 'GET /todos/{id}');
      expect(exporter.spans.single.attributes['http.route'], '/todos/{id}');
    });

    test('gives two ids on one route the same span name', () async {
      final app = build();
      await send(app, '/todos/1');
      await send(app, '/todos/99999');

      expect(exporter.spans.map((span) => span.name).toSet(), hasLength(1));
    });

    test('keeps the path when nothing matched, since no route claimed it',
        () async {
      await send(build(), '/nothing-here');

      expect(exporter.spans.single.name, 'GET /nothing-here');
      expect(exporter.spans.single.attributes, isNot(contains('http.route')));
    });

    test('lets an explicit name win over the matched route', () async {
      // A mounted service doing its own dispatch knows better than the route
      // table, which only sees the mount point.
      await send(
        build(handler: (request) {
          nameSpan('/todos/{id}/custom');
          return {'ok': true};
        }),
        '/todos/7',
      );

      expect(exporter.spans.single.name, '/todos/{id}/custom');
      expect(
        exporter.spans.single.attributes['http.route'],
        '/todos/{id}/custom',
      );
    });

    test('hands out a child span for inner work', () async {
      Span? child;
      await send(
        build(handler: (request) {
          child = CurrentSpan.startChild('db.query');
          return {'ok': true};
        }),
        '/todos/7',
      );

      expect(child!.name, 'db.query');
      expect(child!.context.traceId, exporter.spans.single.context.traceId);
      expect(child!.parentSpanId, exporter.spans.single.context.spanId);
    });

    test('is null outside a traced request', () {
      expect(CurrentSpan.value, isNull);
      expect(CurrentSpan.startChild('x'), isNull);
    });

    test('records nothing when nothing is tracing', () {
      expect(() => CurrentSpan.setAttribute('a', 1), returnsNormally);
      expect(() => nameSpan('GET /x'), returnsNormally);
    });
  });
}
