# Tracing

One span per request, in a format other services already speak.

```dart
import 'package:dust_server/tracing.dart';

final app = Router()
  ..layer(Tracing(exporter, serviceName: 'todos'))
  ..merge(routes);
```

## The wire format is not ours

Spans travel as **W3C Trace Context**: `traceparent` is
`00-<32 hex trace id>-<16 hex span id>-<2 hex flags>`. Inventing a format here
would mean a trace that stops at the first boundary, so a request arriving from
a service written in something else joins the trace it is already part of.

| Incoming | What happens |
| :--- | :--- |
| a valid `traceparent` | this span joins that trace, as a child of the incoming span |
| no header | a new trace starts |
| a malformed header | a new trace starts, and the request is **answered normally** |

A broken collector upstream is an observability problem. Failing the request
over an unreadable trace id would turn it into an outage.

Every response carries the `traceparent` of the span that answered, so a user
can quote it in a bug report and an operator can find the request.

## What a span records

| Attribute | From |
| :--- | :--- |
| `http.request.method` | the request |
| `url.path`, `url.query` | the request; the query is omitted when empty |
| `http.response.status_code` | the response |
| `http.route` | `nameSpan(...)`, when called |
| `service.name` | `Tracing(serviceName:)` |
| `error.type` | an error that escaped the handler |

Names follow OpenTelemetry's semantic conventions where one exists, so a
collector understands them without a mapping.

**5xx is an error; 4xx is not.** A 404 is the server working correctly and
telling the client something true. Marking it failed buries the failures that
matter.

## Name spans after the route, not the path

```dart
nameSpan('GET /todos/{id}');
```

A span named `GET /todos/7` makes every id its own operation, and a dashboard of
a million one-request operations says nothing.

## Reaching the current span

```dart
CurrentSpan.setAttribute('todo.id', id);

final query = CurrentSpan.startChild('db.query');
// ... do the work ...
query?.end(status: SpanStatus.ok);
```

`CurrentSpan` is zone-scoped, like the error sink, so two servers in one
isolate — normal in tests — do not write into each other's traces. Both calls
are no-ops when tracing is not installed, which is what lets a repository
annotate a span without depending on the layer.

## Where spans go

`SpanExporter` is one method, deliberately. **No exporter ships**: binding to
OpenTelemetry, Jaeger, or a log line is an application's dependency, not the
runtime's.

```dart
final class LoggingSpans implements SpanExporter {
  @override
  void export(Span span) {
    print('${span.name} ${span.duration?.inMilliseconds}ms ${span.status}');
  }
}
```

An exporter that throws **cannot fail the request it is reporting on** — the
response is already decided by the time it runs, and a collector being down is
not a reason to fail work that succeeded. `RecordingSpanExporter` keeps spans in
a list, for tests.

## Trying it

[`example/todo_api.dart`](../../packages/dust_server/example/todo_api.dart)
installs the layer with an exporter that prints one line per span, so a trace
survives a hop in a terminal:

```bash
dart run example/todo_api.dart
```

```bash
# a request with no trace: the response carries the span that answered
curl -i -H 'authorization: Bearer todos:read' localhost:8080/api/v1/todos \
  | grep -i traceparent

# a request that is already part of a trace: the same trace id comes back
curl -i -H 'authorization: Bearer todos:read' \
  -H 'traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01' \
  localhost:8080/api/v1/todos | grep -i traceparent

# a malformed header: answered anyway, with a fresh trace
curl -i -H 'authorization: Bearer todos:read' \
  -H 'traceparent: nonsense' \
  localhost:8080/api/v1/todos | grep -i traceparent
```

The second prints a `traceparent` carrying `4bf92f…4736` with a **different**
span id — same trace, new span:

```text
traceparent: 00-7424cdc3b56f26ce133cb44db1c0d272-0d90e1659f296f09-01
traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-7e115a2e956f7b23-01
traceparent: 00-4ae1f0e56ed5209d05cd4b5d3d85271c-4aa3e513ac1f82ed-01
```

The third started a fresh trace and still answered `200`.

The server prints the span as it finishes:

```text
GET /api/v1/todos 3ms ok trace=7424cdc3b56f26ce133cb44db1c0d272
```

## What is still missing

Metrics. There are no counters and no histograms, so "how many" and "how slow"
can only be answered by reading every span. See
[Production readiness](../design/server-production-readiness.md).
