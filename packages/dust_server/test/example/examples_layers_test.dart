import 'dart:io';
import 'package:dust_server/server.dart';
import 'package:test/test.dart';
import '../../example/access_log.dart' as access_log;
import '../../example/compression.dart' as compression;
import '../../example/cors.dart' as cors;
import '../../example/request_id.dart' as request_id;
import 'serve.dart';

/// Layers wrapped around a handler, and the order they run in.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
void main() {
  group('cors', () {
    test('an allowed origin gets the allow header and Vary', () async {
      final app = await example(cors.buildApp());

      final response = await app.get(
        '/api/notes',
        headers: {'origin': 'https://app.example'},
      );

      expect(
        response.headers['access-control-allow-origin'],
        'https://app.example',
      );
      // Without Vary a cache can serve one origin's response to another.
      expect(response.headers['vary'], contains('Origin'));
    });

    test('an origin not on the list gets no allow header', () async {
      final app = await example(cors.buildApp());

      final response = await app.get(
        '/api/notes',
        headers: {'origin': 'https://evil.example'},
      );

      // The body still comes back: CORS instructs the browser, it is not a
      // server-side gate. curl sees the data either way.
      expect(response.statusCode, 200);
      expect(response.headers['access-control-allow-origin'], isNull);
    });

    test('a preflight is answered without reaching the handler', () async {
      final app = await example(cors.buildApp());

      final response = await app.raw(
        'OPTIONS',
        '/api/notes',
        headers: {
          'origin': 'https://app.example',
          'access-control-request-method': 'POST',
        },
      );

      expect(response.statusCode, anyOf(200, 204));
      expect(
        response.headers['access-control-allow-methods'],
        contains('POST'),
      );
      expect(response.headers['access-control-max-age'], '600');
    });

    test('exposeHeaders is what lets fetch read x-request-id', () async {
      final app = await example(cors.buildApp());

      final response = await app.get(
        '/api/notes',
        headers: {'origin': 'https://app.example'},
      );

      expect(
        response.headers['access-control-expose-headers'],
        contains('x-request-id'),
      );
    });

    test('credentials with a wildcard origin throws at construction', () {
      // A browser refuses a credentialed response allowed for "*", so this is
      // caught here rather than in someone console.
      expect(
        () => Cors(origins: const AllowedOrigins.any(), credentials: true),
        throwsArgumentError,
      );
    });
  });

  group('compression', () {
    test('a big enough body is gzipped, and Vary is set', () async {
      final app = await example(compression.buildApp());

      final client = HttpClient()..autoUncompress = false;
      addTearDown(client.close);
      final request = await client.getUrl(app.uri('/rows'));
      request.headers.set('accept-encoding', 'gzip');
      final response = await request.close();
      final bytes = await response.fold<List<int>>(
        <int>[],
        (all, chunk) => all..addAll(chunk),
      );

      expect(response.headers.value('content-encoding'), 'gzip');
      expect(response.headers.value('vary'), contains('Accept-Encoding'));
      expect(gzip.decode(bytes).length, greaterThan(bytes.length));
    });

    test('gzip;q=0 is a refusal, not an absence', () async {
      final app = await example(compression.buildApp());

      final response = await app.get(
        '/rows',
        headers: {'accept-encoding': 'gzip;q=0'},
      );

      expect(response.headers['content-encoding'], isNull);
    });

    test('a body under the threshold is left alone', () async {
      final app = await example(compression.buildApp());

      final response = await app.get(
        '/ping',
        headers: {'accept-encoding': 'gzip'},
      );

      expect(response.body.length, lessThan(1024));
      expect(response.headers['content-encoding'], isNull);
    });
  });

  group('request_id', () {
    test('every answer carries an id', () async {
      final app = await example(request_id.buildApp(log: (_) {}));

      expect((await app.get('/notes')).headers['x-request-id'], isNotEmpty);
    });

    test('a client-supplied id is kept, so one id spans every hop', () async {
      final app = await example(request_id.buildApp(log: (_) {}));

      final response = await app.get(
        '/notes',
        headers: {'x-request-id': 'from-the-gateway'},
      );

      expect(response.headers['x-request-id'], 'from-the-gateway');
    });

    test('the handler reads the same id the response carries', () async {
      final app = await example(request_id.buildApp(log: (_) {}));

      final response = await app.get(
        '/echo-id',
        headers: {'x-request-id': 'abc-123'},
      );

      expect(app.object(response), {'requestId': 'abc-123'});
      expect(response.headers['x-request-id'], 'abc-123');
    });

    test('two requests get different ids', () async {
      final app = await example(request_id.buildApp(log: (_) {}));

      final first = (await app.get('/notes')).headers['x-request-id'];
      final second = (await app.get('/notes')).headers['x-request-id'];

      expect(first, isNot(second));
    });
  });

  group('access_log', () {
    test('records the method, path, and status', () async {
      final records = <AccessRecord>[];
      final app = await example(access_log.buildApp(onRecord: records.add));

      await app.get('/notes');

      expect(records.single.method, 'GET');
      expect(records.single.path, '/notes');
      expect(records.single.status, 200);
    });

    test('records a 404 too, because it is a request', () async {
      // Above the routes on purpose: a request that never reaches the log is
      // one nobody can explain.
      final records = <AccessRecord>[];
      final app = await example(access_log.buildApp(onRecord: records.add));

      await app.get('/nothing');

      expect(records.single.status, 404);
      expect(records.single.path, '/nothing');
    });

    test('the recorded path carries no query string', () async {
      // Query strings carry API keys and reset tokens. An access log is a
      // recognised place they leak.
      final records = <AccessRecord>[];
      final app = await example(access_log.buildApp(onRecord: records.add));

      await app.get('/notes?api_key=secret');

      expect(records.single.path, '/notes');
      expect(records.single.path, isNot(contains('secret')));
    });

    test('the record carries the request id, so the two logs join up',
        () async {
      final records = <AccessRecord>[];
      final app = await example(access_log.buildApp(onRecord: records.add));

      await app.get('/notes', headers: {'x-request-id': 'abc-123'});

      expect(records.single.requestId, 'abc-123');
    });
  });
}
