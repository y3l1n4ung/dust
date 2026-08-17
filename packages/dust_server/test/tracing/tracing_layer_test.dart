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

void main() {
  const traceId = '4bf92f3577b34da6a3ce929d0e0e4736';
  const spanId = '00f067aa0ba902b7';

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

  group('a request with no trace', () {
    test('records one span', () async {
      await send(build(), '/todos/7');

      expect(exporter.spans, hasLength(1));
    });

    test('starts a trace of its own', () async {
      await send(build(), '/todos/7');

      expect(TraceContext.parse(exporter.spans.single.context.traceparent),
          isNotNull);
    });

    test('has no parent', () async {
      await send(build(), '/todos/7');

      expect(exporter.spans.single.parentSpanId, isNull);
    });

    test('carries the method and path', () async {
      await send(build(), '/todos/7?full=1');

      expect(
          exporter.spans.single.attributes,
          containsPair(
            'http.request.method',
            'GET',
          ));
      expect(exporter.spans.single.attributes['url.path'], '/todos/7');
      expect(exporter.spans.single.attributes['url.query'], 'full=1');
    });

    test('omits the query when there is none', () async {
      await send(build(), '/todos/7');

      expect(exporter.spans.single.attributes, isNot(contains('url.query')));
    });

    test('records the service when it was named', () async {
      await send(build(serviceName: 'todos'), '/todos/7');

      expect(exporter.spans.single.attributes['service.name'], 'todos');
    });
  });

  group('a request carrying a trace', () {
    test('joins it rather than starting a new one', () async {
      await send(
        build(),
        '/todos/7',
        headers: const {'traceparent': '00-$traceId-$spanId-01'},
      );

      expect(exporter.spans.single.context.traceId, traceId);
    });

    test('becomes a child of the incoming span', () async {
      await send(
        build(),
        '/todos/7',
        headers: const {'traceparent': '00-$traceId-$spanId-01'},
      );

      expect(exporter.spans.single.parentSpanId, spanId);
    });

    test('takes a span id of its own', () async {
      await send(
        build(),
        '/todos/7',
        headers: const {'traceparent': '00-$traceId-$spanId-01'},
      );

      expect(exporter.spans.single.context.spanId, isNot(spanId));
    });

    test('inherits the sampling decision', () async {
      await send(
        build(),
        '/todos/7',
        headers: const {'traceparent': '00-$traceId-$spanId-00'},
      );

      expect(exporter.spans.single.context.sampled, isFalse);
    });

    test('starts a new trace when the header is malformed', () async {
      await send(
        build(),
        '/todos/7',
        headers: const {'traceparent': 'nonsense'},
      );

      expect(exporter.spans.single.context.traceId, isNot(traceId));
      expect(exporter.spans.single.parentSpanId, isNull);
    });

    test('answers rather than failing on a malformed header', () async {
      final response = await send(
        build(),
        '/todos/7',
        headers: const {'traceparent': 'nonsense'},
      );

      expect(response.statusCode, 200);
    });
  });

  group('the response', () {
    test('carries the span that answered', () async {
      final response = await send(build(), '/todos/7');

      expect(
        response.headers['traceparent'],
        exporter.spans.single.context.traceparent,
      );
    });

    test('carries the trace the caller started', () async {
      final response = await send(
        build(),
        '/todos/7',
        headers: const {'traceparent': '00-$traceId-$spanId-01'},
      );

      expect(response.headers['traceparent'], contains(traceId));
    });
  });

  group('status', () {
    test('is ok for a 200', () async {
      await send(build(), '/todos/7');

      expect(exporter.spans.single.status, SpanStatus.ok);
    });

    test('is ok for a 404, which is the server working', () async {
      await send(build(), '/missing');

      expect(exporter.spans.single.status, SpanStatus.ok);
      expect(
        exporter.spans.single.attributes['http.response.status_code'],
        404,
      );
    });

    test('is an error for a 500', () async {
      await send(build(), '/boom');

      expect(exporter.spans.single.status, SpanStatus.error);
    });

    test('records how long the request took', () async {
      await send(build(), '/todos/7');

      expect(exporter.spans.single.duration, isNotNull);
      expect(exporter.spans.single.isFinished, isTrue);
    });
  });

  group('an error escaping past the layer', () {
    // The verb builder normally turns a throw into a 500 before the layer sees
    // it. A plain `shelf` handler underneath has no such guard, which is the
    // case this covers.
    Handler raw() => Tracing(exporter).toMiddleware()(
          (request) => throw StateError('boom'),
        );

    test('is rethrown rather than swallowed', () async {
      expect(
        () => Future.sync(() => raw()(request('GET', '/x'))),
        throwsA(isA<StateError>()),
      );
    });

    test('still records the span', () async {
      try {
        await Future.sync(() => raw()(request('GET', '/x')));
      } on StateError {
        // expected
      }

      expect(exporter.spans.single.status, SpanStatus.error);
    });

    test('records what kind of error it was', () async {
      try {
        await Future.sync(() => raw()(request('GET', '/x')));
      } on StateError {
        // expected
      }

      expect(exporter.spans.single.attributes['error.type'], 'StateError');
    });
  });

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
