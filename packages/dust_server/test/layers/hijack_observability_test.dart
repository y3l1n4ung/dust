import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// What the observability layers make of a WebSocket upgrade.
///
/// An upgrade *succeeds* by throwing `HijackException`, which is the one place
/// a throw does not mean failure. A layer that treats it as one gets the answer
/// exactly backwards: the connections that worked are the ones marked broken.
///
/// Both of these were wrong. Tracing recorded every successful upgrade as an
/// error, so a chat server read as 100% failure on the endpoint that worked.
/// The access log recorded nothing at all, so a connection was invisible —
/// nobody could see who connected, when, or from where.

final class _Exporter implements SpanExporter {
  const _Exporter(this.into);

  final List<Span> into;

  @override
  void export(Span span) => into.add(span);
}

/// Serves an upgrade route and an ordinary one, behind both layers.
Future<ServerHandle> serve(
  List<Span> spans,
  List<AccessRecord> records,
) {
  final app = Router()
    ..layer(Tracing(_Exporter(spans), serviceName: 'test'))
    ..layer(AccessLog(records.add))
    ..route('/ws', ws((session) async => session.close()))
    ..route('/plain', get((request) async => const {'ok': true}));

  return serveRouter(app, InternetAddress.loopbackIPv4, 0);
}

void main() {
  group('a WebSocket upgrade', () {
    late List<Span> spans;
    late List<AccessRecord> records;
    late ServerHandle server;

    setUp(() async {
      spans = [];
      records = [];
      server = await serve(spans, records);
    });

    tearDown(() => server.close(drain: const Duration(seconds: 1)));

    Future<void> connect() async {
      final socket =
          await WebSocket.connect('ws://127.0.0.1:${server.port}/ws');
      await socket.close();
      // The layers record on the way out of the throw, which happens after the
      // handshake the client already saw.
      await Future<void>.delayed(const Duration(milliseconds: 50));
    }

    test('is traced as a success, not an error', () async {
      await connect();

      expect(spans.single.status, SpanStatus.ok);
      expect(spans.single.attributes.containsKey('error.type'), isFalse);
    });

    test('is traced with 101, which is what went on the wire', () async {
      await connect();

      expect(spans.single.attributes['http.response.status_code'], 101);
    });

    test('reaches the access log', () async {
      await connect();

      expect(records.single.method, 'GET');
      expect(records.single.path, '/ws');
      expect(records.single.status, 101);
    });

    test('carries its matched route into the log, like any request', () async {
      await connect();

      expect(records.single.matchedRoute, '/ws');
    });

    test('does not disturb an ordinary request beside it', () async {
      await connect();

      final client = HttpClient();
      addTearDown(client.close);
      final response = await (await client
              .getUrl(Uri.parse('http://127.0.0.1:${server.port}/plain')))
          .close();
      await response.drain<void>();
      await Future<void>.delayed(const Duration(milliseconds: 50));

      expect(records.map((record) => record.status), [101, 200]);
      expect(spans.map((span) => span.status), [
        SpanStatus.ok,
        SpanStatus.ok,
      ]);
    });

    test('a genuine failure is still an error span', () async {
      // The fix must not make every throw look like a success.
      final failures = <Span>[];
      final logged = <AccessRecord>[];
      final failing = await serveRouter(
        Router(onError: (_, __) {})
          ..layer(Tracing(_Exporter(failures), serviceName: 'test'))
          ..layer(AccessLog(logged.add))
          ..route('/boom', get((request) async => throw StateError('broken'))),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => failing.close(drain: const Duration(seconds: 1)));

      final client = HttpClient();
      addTearDown(client.close);
      final response = await (await client
              .getUrl(Uri.parse('http://127.0.0.1:${failing.port}/boom')))
          .close();
      await response.drain<void>();

      expect(response.statusCode, 500);
      expect(logged.single.status, 500);
      expect(failures.single.status, SpanStatus.error);
    });
  });
}
