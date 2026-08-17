import 'dart:async';

import 'package:shelf/shelf.dart';

import '../request/request_parts.dart';
import '../router/middleware.dart';
import 'span.dart';
import 'trace_context.dart';

/// The span the current request is inside.
///
/// Scoped to a zone rather than to the process, the same way the error sink
/// is, so two servers in one isolate — normal in tests — do not write into
/// each other's traces.
abstract final class CurrentSpan {
  static const _zoneKey = #dust_server.currentSpan;

  /// The span of the request being handled, or `null` outside one.
  static Span? get value {
    final scoped = Zone.current[_zoneKey];
    return scoped is Span ? scoped : null;
  }

  /// Runs [body] with [span] as the current one.
  static R runWith<R>(Span span, R Function() body) =>
      runZoned(body, zoneValues: {_zoneKey: span});

  /// Records an attribute on the current span, if there is one.
  ///
  /// Safe to call from anywhere, including code that does not know whether
  /// tracing is installed — which is what lets a repository annotate a span
  /// without taking a dependency on the layer.
  static void setAttribute(String key, Object? attribute) =>
      value?.setAttribute(key, attribute);

  /// Starts a child of the current span, or `null` when nothing is tracing.
  ///
  /// The caller ends it and exports it; this only builds it, so a database
  /// layer can time a query without knowing where spans go.
  static Span? startChild(String name) => value?.child(name);
}

/// Records one span per request, and continues an incoming trace.
///
/// ```dart
/// final app = Router()
///   ..layer(Tracing(exporter))
///   ..merge(routes);
/// ```
///
/// A request arriving with a `traceparent` joins that trace as a child span;
/// one arriving without starts a new trace. Either way the response carries
/// the `traceparent` of the span that answered, so a client can quote it in a
/// bug report and an operator can find the request.
///
/// The span is named `GET /todos/{id}` — the **route**, not the path — because
/// naming it after the path makes every id its own operation and a dashboard
/// of a million one-request operations says nothing. The route is only known
/// after matching, so the name is corrected once the handler returns, from
/// `matchedPathOf`. A request that reached the fallback keeps its path, since
/// no route claimed it.
final class Tracing implements Layer {
  /// Exports finished spans to [exporter].
  const Tracing(this.exporter, {this.serviceName});

  /// Where finished spans go.
  final SpanExporter exporter;

  /// Recorded as `service.name` when given.
  final String? serviceName;

  @override
  Middleware toMiddleware() {
    return (Handler inner) {
      return (Request request) async {
        final parts = RequestParts.of(request);
        final parent = TraceContext.parse(parts.headers['traceparent']);

        final span = Span(
          name: '${parts.method} ${request.requestedUri.path}',
          context: parent?.child() ?? TraceContext.start(),
          parentSpanId: parent?.spanId,
          attributes: {
            'http.request.method': parts.method,
            'url.path': request.requestedUri.path,
            if (request.requestedUri.query.isNotEmpty)
              'url.query': request.requestedUri.query,
            if (serviceName != null) 'service.name': serviceName,
          },
        );

        // The router fills this on the way through; a layer wraps the matcher,
        // so it cannot read the route off its own request.
        final matched = MatchedRouteSlot();
        final traced = request.change(
          context: {matchedRouteSlotKey: matched},
        );

        try {
          final response = await CurrentSpan.runWith(
            span,
            () => Future.sync(() => inner(traced)),
          );

          // The route is only known once the matcher has run, so the span is
          // renamed here rather than guessed up front. An explicit `nameSpan`
          // wins: a handler that said what it is doing knows better than the
          // route table, which is the case for a mounted service.
          if (matched.route case final route?
              when !span.attributes.containsKey('http.route')) {
            span
              ..name = '${parts.method} $route'
              ..setAttribute('http.route', route);
          }

          span
            ..setAttribute('http.response.status_code', response.statusCode)
            // A 4xx is the server working correctly, so only 5xx is an error;
            // marking 404 as failure buries the failures that matter.
            ..end(
                status: response.statusCode >= 500
                    ? SpanStatus.error
                    : SpanStatus.ok);

          _export(span);
          return response.change(
            headers: {'traceparent': span.context.traceparent},
          );
        } on Object catch (error) {
          span
            ..setAttribute('error.type', error.runtimeType.toString())
            ..end(status: SpanStatus.error);
          _export(span);
          rethrow;
        }
      };
    };
  }

  /// Exports [span], swallowing whatever the exporter does.
  ///
  /// The response is already decided by this point; a collector being down is
  /// not a reason to fail a request that succeeded.
  void _export(Span span) {
    try {
      exporter.export(span);
    } on Object {
      // Deliberately ignored; see above.
    }
  }
}

/// Overrides the route the current span is labelled with.
///
/// The layer already names a span after the route that matched, so this is
/// only for the cases the route table cannot describe — a mounted service that
/// does its own dispatch, or a handler serving several logical operations:
///
/// ```dart
/// nameSpan('/legacy/{action}');
/// ```
///
/// An explicit name wins over the matched route, and does nothing when
/// tracing is not installed.
void nameSpan(String route) {
  final span = CurrentSpan.value;
  if (span == null) return;
  span
    ..name = route
    ..setAttribute('http.route', route);
}
