/// Tracing: one span per request, and a trace that survives a hop.
///
/// ```dart
/// import 'package:dust_server/tracing.dart';
///
/// final app = Router()
///   ..layer(Tracing(exporter, serviceName: 'todos'))
///   ..merge(routes);
/// ```
///
/// The wire format is W3C Trace Context, so a request arriving with a
/// `traceparent` joins that trace and the response carries the span that
/// answered. Where spans go is a `SpanExporter`, which this package does not
/// implement: binding to a collector is an application's dependency, not the
/// runtime's.
///
/// Everything here is also exported from `package:dust_server/server.dart`.
library;

export 'src/tracing/tracing.dart';
