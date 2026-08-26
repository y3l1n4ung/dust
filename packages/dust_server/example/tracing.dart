import 'dart:io';

import 'package:dust_server/server.dart';

/// Spans, and continuing a trace that started upstream.
///
/// One span per request, named after the **matched route** rather than the URL.
/// That distinction is the difference between a usable trace and an unusable
/// one: `GET /orders/{id}` aggregates, `GET /orders/41` gives you one span per
/// order and a dashboard nothing can group.
///
/// `traceparent` is what joins the hops. An incoming one is continued — same
/// trace id, this span's parent set to the caller's — so a request through a
/// gateway and three services is one trace instead of four. An absent one starts
/// a trace.
///
/// `CurrentSpan.setAttribute` reaches the span from anywhere inside the request
/// without threading it through every function, because it is zone-scoped. Use
/// it for the few facts that make a slow trace explicable — which tenant, how
/// many rows, whether a cache was hit.
///
/// > **Attributes go to your collector, and stay there.** They are not the place
/// > for a token, a password, a full request body, or an email address; a trace
/// > backend is rarely as locked down as your database, and it retains for weeks.
/// > Record an id, not the record.
///
/// Run it with `dart run example/tracing.dart`:
///
/// ```bash
/// curl -s localhost:8080/orders/41
/// curl -s localhost:8080/orders/42          # the same span name
/// curl -s localhost:8080/nothing            # a 404 is traced too
/// curl -s localhost:8080/orders/41 \
///   -H 'traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01'
/// ```
///
/// Each prints one line per span on the server's stdout.
Future<void> main() async {
  final server = await serve(
    buildApp(exporter: const PrintSpans()),
    InternetAddress.anyIPv4,
    8080,
  );
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp({SpanExporter? exporter}) {
  return Router()
    // Above the routes, so a 404 is traced too. A request missing from the
    // trace is one nobody can explain.
    ..layer(Tracing(exporter ?? const PrintSpans(), serviceName: 'orders'))
    ..route('/orders/{id}', get(readOrder))
    ..route('/legacy/{action}', get(legacy));
}

/// `GET /orders/{id}`
Future<Map<String, Object?>> readOrder(Request request) async {
  final id = await request.path<int>('id');

  // An id, not the customer. Attributes leave the process.
  CurrentSpan.setAttribute('order.id', id);
  CurrentSpan.setAttribute('cache.hit', false);

  return {'id': id};
}

/// `GET /legacy/{action}` — a route whose pattern is not the useful name.
///
/// `nameSpan` overrides what the layer would have used. Worth it when one route
/// serves several logical operations, and wrong when it just hides the route.
Future<Map<String, Object?>> legacy(Request request) async {
  final action = await request.path<String>('action');

  nameSpan('legacy.$action');

  return {'action': action};
}

/// Writes one line per span, which is enough to read a trace in a terminal.
final class PrintSpans implements SpanExporter {
  /// Creates the exporter.
  const PrintSpans();

  @override
  void export(Span span) {
    stdout.writeln(
      '${span.name} ${span.duration?.inMilliseconds}ms '
      'trace=${span.context.traceId} parent=${span.parentSpanId ?? '-'} '
      '${span.attributes}',
    );
  }
}
