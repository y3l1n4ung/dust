import 'trace_context.dart';

/// How a span finished.
enum SpanStatus {
  /// Still running.
  unset,

  /// Finished the way it was meant to.
  ok,

  /// Failed. For a request span this means 5xx, not 4xx: a 404 is the server
  /// working correctly and telling the client something true.
  error,
}

/// One timed operation inside a trace.
///
/// Mutable on purpose. A handler adds attributes to the span it is already
/// inside while the work happens, and forcing a rebuild for each one would put
/// plumbing in every handler that wants to record a customer id.
final class Span {
  /// Starts a span named [name] in [context].
  Span({
    required this.name,
    required this.context,
    this.parentSpanId,
    DateTime? startedAt,
    Map<String, Object?>? attributes,
  })  : startedAt = startedAt ?? DateTime.now().toUtc(),
        attributes = attributes ?? <String, Object?>{};

  /// What this span measures: `GET /todos/{id}`, `db.query`.
  ///
  /// Mutable because a request span cannot be named correctly until the router
  /// has matched: the route is what belongs here, and it is not known when the
  /// span starts.
  String name;

  /// The trace and span ids.
  final TraceContext context;

  /// The span this one is inside, or `null` at the root of a trace.
  final String? parentSpanId;

  /// When it started, in UTC.
  final DateTime startedAt;

  /// When it finished, or `null` while it is running.
  DateTime? endedAt;

  /// How it finished.
  SpanStatus status = SpanStatus.unset;

  /// What was recorded about it.
  ///
  /// Keys follow OpenTelemetry's semantic conventions where one exists —
  /// `http.request.method`, `http.response.status_code`, `http.route` — so a
  /// collector understands them without a mapping.
  final Map<String, Object?> attributes;

  /// How long it ran, or `null` while it is running.
  Duration? get duration => endedAt?.difference(startedAt);

  /// Whether it has finished.
  bool get isFinished => endedAt != null;

  /// Records one attribute.
  void setAttribute(String key, Object? value) => attributes[key] = value;

  /// Records several attributes at once.
  void setAttributes(Map<String, Object?> values) => attributes.addAll(values);

  /// Marks this span finished.
  ///
  /// Calling it twice keeps the first end time: the layer ends the span when
  /// the handler returns, and a handler that already ended it meant that time.
  void end({DateTime? at, SpanStatus? status}) {
    if (status != null) this.status = status;
    endedAt ??= at ?? DateTime.now().toUtc();
  }

  /// Starts a child span of this one, in the same trace.
  Span child(String name) => Span(
        name: name,
        context: context.child(),
        parentSpanId: context.spanId,
      );

  @override
  String toString() =>
      'Span($name, ${context.traceId}/${context.spanId}, $status)';
}

/// Where finished spans go.
///
/// One method, so a binding to OpenTelemetry, Jaeger, a log line, or a test
/// list is a class with one override. Nothing in this package talks to a
/// collector; that stays an application's choice and an application's
/// dependency.
///
/// ```dart
/// final class LoggingSpans implements SpanExporter {
///   @override
///   void export(Span span) => print('${span.name} ${span.duration}');
/// }
/// ```
abstract interface class SpanExporter {
  /// Called once per span, after it finished.
  ///
  /// Must not throw: the request is already answered, and an exporter failing
  /// should not turn into a handler error.
  void export(Span span);
}

/// A [SpanExporter] that keeps spans in a list, for tests.
final class RecordingSpanExporter implements SpanExporter {
  /// Creates an exporter that records into [spans].
  RecordingSpanExporter([List<Span>? spans]) : spans = spans ?? <Span>[];

  /// The spans exported so far, in the order they finished.
  final List<Span> spans;

  @override
  void export(Span span) => spans.add(span);
}
