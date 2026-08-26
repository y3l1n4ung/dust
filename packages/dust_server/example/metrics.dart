import 'dart:io';

import 'package:dust_server/server.dart';

/// Counting requests and timing them.
///
/// The runtime has no metrics of its own, on purpose: what to count, how to
/// bucket it, and what to call it are decisions only the application can make,
/// and a built-in set that does not match your dashboard is worse than none.
/// What it gives you is `AccessRecord`, which carries everything a counter needs
/// — including the matched route.
///
/// > **Label by matched route, never by URL.** `record.matchedRoute` is
/// > `/orders/{id}`; `record.path` is `/orders/41`, `/orders/42`, and one time
/// > series per order. That is a cardinality explosion, and it is the most common
/// > way a Prometheus server or a metrics bill falls over.
///
/// The same rule kills two more temptations: do not label by user id, and do not
/// label by anything a client controls. A caller who can invent label values can
/// invent unbounded ones — and an unmatched path is client-controlled, which is
/// why every 404 collapses to one series here.
///
/// Prometheus text format, because a scrape endpoint needs no client library and
/// no push infrastructure.
///
/// Run it with `dart run example/metrics.dart`:
///
/// ```bash
/// curl -s localhost:8080/orders/41 > /dev/null
/// curl -s localhost:8080/orders/42 > /dev/null
/// curl -s localhost:8080/nothing   > /dev/null
/// curl -s localhost:8080/metrics
/// ```
///
/// Two requests to different orders share one series. That is the point.
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp({Metrics? metrics}) {
  final collected = metrics ?? Metrics();

  return Router()
    // Above the routes, so 404s and 405s are counted too. A metric that only
    // sees successes cannot show an outage.
    ..layer(AccessLog(collected.record))
    ..route('/orders/{id}', get(readOrder))
    ..route('/metrics', get(scrape))
    ..withState(collected);
}

/// `GET /orders/{id}`
Future<Map<String, Object?>> readOrder(Request request) async => {
      'id': await request.path<int>('id'),
    };

/// `GET /metrics` — the scrape endpoint.
///
/// In production this belongs on a separate port or behind a network rule.
/// Request counts and latencies are reconnaissance: they show which endpoints
/// exist, which are slow, and when a deploy happened.
Future<Response> scrape(Request request) async {
  final metrics = await request.state<Metrics>();

  return textResponse(metrics.render());
}

/// A counter and a latency histogram, labelled by route.
final class Metrics {
  /// Counts, keyed by route, method, and status.
  final counts = <String, int>{};

  /// Latency bucket tallies, keyed the same way.
  final buckets = <String, List<int>>{};

  /// Bucket boundaries, in milliseconds.
  static const boundaries = [5, 25, 100, 500, 2000];

  /// Folds one request into the counters.
  void record(AccessRecord entry) {
    // The matched route, not the path. A 404 has none, and collapsing those to
    // one label keeps a scanner walking your URLs from creating a series each.
    final route = entry.matchedRoute ?? '<unmatched>';
    final key = '$route ${entry.method} ${entry.status}';

    counts[key] = (counts[key] ?? 0) + 1;
    final histogram = buckets[key] ??= List.filled(boundaries.length + 1, 0);
    histogram[_bucketFor(entry.duration)]++;
  }

  /// The index of the bucket [duration] falls in.
  int _bucketFor(Duration duration) {
    final milliseconds = duration.inMilliseconds;
    for (var index = 0; index < boundaries.length; index++) {
      if (milliseconds <= boundaries[index]) return index;
    }
    return boundaries.length;
  }

  /// Prometheus text format.
  String render() {
    final out = StringBuffer()
      ..writeln('# TYPE http_requests_total counter')
      ..writeln('# TYPE http_request_duration_ms histogram');

    for (final entry in counts.entries) {
      final parts = entry.key.split(' ');
      final labels = 'route="${parts[0]}",method="${parts[1]}",'
          'status="${parts[2]}"';
      out.writeln('http_requests_total{$labels} ${entry.value}');

      // Prometheus histogram buckets are cumulative: each le= counts everything
      // at or below it, not just its own slice.
      var cumulative = 0;
      final histogram = buckets[entry.key]!;
      for (var index = 0; index < boundaries.length; index++) {
        cumulative += histogram[index];
        out.writeln(
          'http_request_duration_ms_bucket{$labels,'
          'le="${boundaries[index]}"} $cumulative',
        );
      }
      cumulative += histogram.last;
      out.writeln(
        'http_request_duration_ms_bucket{$labels,le="+Inf"} $cumulative',
      );
    }

    return out.toString();
  }
}
