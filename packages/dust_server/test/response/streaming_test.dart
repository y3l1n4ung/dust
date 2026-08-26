import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// Whether a streamed response actually reaches the client incrementally.
///
/// This is the one property no status-code assertion can catch. `shelf` buffers
/// a streamed body by default and flushes it when the stream ends — so an SSE
/// response looked correct in every other test while delivering all of its
/// events at once, and a stream that never ends delivered nothing at all.
///
/// The tests below read a raw socket and record *when* bytes arrive, because
/// that is the only way to tell a stream from a buffer.

/// Connects, sends a `GET`, and records the elapsed time of each arrival.
Future<List<int>> arrivalsFor(ServerHandle server, String path) async {
  final socket = await Socket.connect('127.0.0.1', server.port);
  final started = Stopwatch()..start();

  socket.write('GET $path HTTP/1.1\r\n'
      'Host: localhost\r\n'
      'accept-encoding: identity\r\n'
      'Connection: close\r\n\r\n');
  await socket.flush();

  final arrivals = <int>[];
  await for (final chunk in socket) {
    if (chunk.isNotEmpty) arrivals.add(started.elapsedMilliseconds);
  }
  await socket.close();
  return arrivals;
}

/// Five events, one every 60ms.
Response ticks(Request request) => eventStream(
      Stream<ServerSentEvent>.periodic(
        const Duration(milliseconds: 60),
        (index) => ServerSentEvent(data: 'tick $index', id: '$index'),
      ).take(5),
      keepAlive: null,
    );

/// Five chunks, one every 60ms.
Stream<List<int>> slowChunks() async* {
  for (var index = 0; index < 5; index++) {
    await Future<void>.delayed(const Duration(milliseconds: 60));
    yield utf8.encode('chunk $index\n');
  }
}

void main() {
  group('a handler that returns a stream', () {
    test('is served rather than failing to encode', () async {
      // It used to reach `jsonEncode`, which cannot encode a Stream, so the
      // natural way to send a large file answered 500.
      final server = await serve(
        Router()..route('/download', get((request) async => slowChunks())),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final client = HttpClient();
      addTearDown(client.close);
      final response = await (await client
              .getUrl(Uri.parse('http://127.0.0.1:${server.port}/download')))
          .close();
      final body = await utf8.decodeStream(response);

      expect(response.statusCode, 200);
      expect(body, contains('chunk 0'));
      expect(body, contains('chunk 4'));
    });

    test('reaches the client as it is produced', () async {
      final server = await serve(
        Router()..route('/download', get((request) async => slowChunks())),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final arrivals = await arrivalsFor(server, '/download');

      expect(arrivals.length, greaterThanOrEqualTo(5));
      expect(arrivals.last - arrivals.first, greaterThan(150));
    });
  });

  group('a response built by hand', () {
    test('streams even though it never opted out of buffering', () async {
      // `Response.ok(stream)` is plain shelf, which buffers by default. Serving
      // turns that off when the length is unknown, so a hand-built response and
      // a mounted third-party handler stream like everything else.
      final server = await serve(
        Router()
          ..route('/raw', get((request) async => Response.ok(slowChunks()))),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final arrivals = await arrivalsFor(server, '/raw');

      expect(arrivals.length, greaterThanOrEqualTo(5));
      expect(arrivals.last - arrivals.first, greaterThan(150));
    });

    test('a mounted handler streams too', () async {
      // Whatever is mounted has never heard of Dust and cannot opt in itself.
      final server = await serve(
        Router()..mount('/', (request) async => Response.ok(slowChunks())),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      expect((await arrivalsFor(server, '/anything')).length,
          greaterThanOrEqualTo(5));
    });

    test('asking for buffering is still respected', () async {
      // Worth having for a body that arrives as very many tiny chunks, where
      // one write each is slower than one write.
      final server = await serve(
        Router()
          ..route(
            '/buffered',
            get((request) async => Response.ok(
                  slowChunks(),
                  context: const {'shelf.io.buffer_output': true},
                )),
          ),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final arrivals = await arrivalsFor(server, '/buffered');

      // Headers, then the whole body at the end.
      expect(arrivals.length, lessThan(4));
    });

    test('a response with a known length is left alone', () async {
      // One write either way, so buffering costs nothing and helps.
      final server = await serve(
        Router()..route('/json', get((request) async => const {'ok': true})),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final client = HttpClient();
      addTearDown(client.close);
      final response = await (await client
              .getUrl(Uri.parse('http://127.0.0.1:${server.port}/json')))
          .close();

      expect(response.headers.contentLength, greaterThan(0));
      expect(await utf8.decodeStream(response), '{"ok":true}');
    });
  });

  group('the streamed helper', () {
    test('sets the content type it was given', () async {
      final response = streamed(
        const Stream<List<int>>.empty(),
        contentType: 'text/csv',
      );

      expect(response.headers['content-type'], 'text/csv');
      expect(response.context['shelf.io.buffer_output'], false);
    });

    test('sets no content-length, so it can start before the work ends',
        () async {
      final response = streamed(const Stream<List<int>>.empty());

      expect(response.headers['content-length'], isNull);
    });

    test('carries extra headers and a status through', () async {
      final response = streamed(
        const Stream<List<int>>.empty(),
        status: 206,
        headers: const {'content-range': 'bytes 0-99/200'},
      );

      expect(response.statusCode, 206);
      expect(response.headers['content-range'], 'bytes 0-99/200');
    });

    test('delivers incrementally', () async {
      final server = await serve(
        Router()
          ..route(
            '/csv',
            get((request) async =>
                streamed(slowChunks(), contentType: 'text/csv')),
          ),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final arrivals = await arrivalsFor(server, '/csv');

      expect(arrivals.length, greaterThanOrEqualTo(5));
    });
  });

  group('an event stream', () {
    test('delivers its events as they happen, not all at the end', () async {
      final server = await serve(
        Router()..route('/events', get(ticks)),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final arrivals = await arrivalsFor(server, '/events');

      // Headers, then one arrival per event. Buffered, this was two arrivals:
      // the headers and then everything at once.
      expect(arrivals.length, greaterThanOrEqualTo(5));

      // The gap between the first and last event is roughly the stream's own
      // duration. Buffered, they were milliseconds apart.
      expect(arrivals.last - arrivals.first, greaterThan(150));
    });

    test('the first event does not wait for the last', () async {
      final server = await serve(
        Router()..route('/events', get(ticks)),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final socket = await Socket.connect('127.0.0.1', server.port);
      final started = Stopwatch()..start();
      socket.write('GET /events HTTP/1.1\r\nHost: localhost\r\n'
          'accept-encoding: identity\r\nConnection: close\r\n\r\n');
      await socket.flush();

      final firstEvent = Completer<int>();
      final subscription = socket.listen((chunk) {
        if (utf8.decode(chunk, allowMalformed: true).contains('tick 0') &&
            !firstEvent.isCompleted) {
          firstEvent.complete(started.elapsedMilliseconds);
        }
      });
      addTearDown(subscription.cancel);

      final at = await firstEvent.future.timeout(const Duration(seconds: 2));

      // The whole stream runs for ~300ms. The first event has to beat that by a
      // wide margin, or it was held until the end.
      expect(at, lessThan(200));
      await socket.close();
    });

    test('says so in the response context, which is what the adapter reads',
        () async {
      final response = eventStream(const Stream<ServerSentEvent>.empty());

      expect(response.context['shelf.io.buffer_output'], false);
    });

    test('still sets the headers that stop a proxy buffering', () async {
      // Two different buffers: this process, and whatever is in front of it.
      final response = eventStream(const Stream<ServerSentEvent>.empty());

      expect(response.headers['x-accel-buffering'], 'no');
      expect(response.headers['cache-control'], 'no-cache');
    });

    test('streams through the compression layer too', () async {
      // Compression is allowed to gzip an event stream, but it must not turn it
      // back into a buffer while doing so.
      final server = await serve(
        Router()
          ..layer(const Compression())
          ..route('/events', get(ticks)),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => server.close(drain: const Duration(seconds: 1)));

      final arrivals = await arrivalsFor(server, '/events');

      expect(arrivals.length, greaterThanOrEqualTo(5));
      expect(arrivals.last - arrivals.first, greaterThan(150));
    });
  });
}
