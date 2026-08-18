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

void main() {
  group('an event stream', () {
    test('delivers its events as they happen, not all at the end', () async {
      final server = await serveRouter(
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
      final server = await serveRouter(
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
      final server = await serveRouter(
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
