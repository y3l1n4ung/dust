import 'dart:io';
import 'package:test/test.dart';
import '../../example/redirects.dart' as redirects;
import '../../example/sse.dart' as sse;
import '../../example/websockets.dart' as websockets;
import 'serve.dart';

/// What a handler sends back, including streams and sockets.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
void main() {
  group('redirects', () {
    test('a POST answers 303, so a reload does not re-submit', () async {
      final app = await example(redirects.buildApp());

      final response = await app.raw('POST', '/notes');

      expect(response.statusCode, 303);
      expect(response.headers['location'], '/notes/1');
    });

    test('permanent is 308, which keeps the method', () async {
      final app = await example(redirects.buildApp());

      final response = await app.raw('GET', '/old-home');

      expect(response.statusCode, 308);
      expect(response.headers['location'], '/');
    });

    test('temporary is 307, so nothing caches it', () async {
      final app = await example(redirects.buildApp());

      expect((await app.raw('GET', '/maintenance')).statusCode, 307);
    });

    test('a newline in the target is stripped from Location', () async {
      // Location is one of the few places caller input reaches a header, and a
      // newline there would let a client inject headers of its own.
      final app = await example(redirects.buildApp());

      final response = await app.raw(
        'GET',
        '/search?q=a%0d%0aX-Evil%3A%201',
      );

      expect(response.statusCode, 303);
      expect(response.headers['location'], isNot(contains('\n')));
      expect(response.headers['location'], isNot(contains('\r')));
      expect(response.headers['x-evil'], isNull);
    });
  });

  group('sse', () {
    test('sets the three headers a stream needs', () async {
      final app = await example(sse.buildApp());

      final response = await app.get('/ticks');

      expect(response.headers['content-type'], startsWith('text/event-stream'));
      expect(response.headers['cache-control'], 'no-cache');
      // Without this nginx buffers, and the stream looks like a hung server.
      expect(response.headers['x-accel-buffering'], 'no');
    });

    test('each event carries its data and id', () async {
      final app = await example(sse.buildApp());

      final body = (await app.get('/ticks')).body;

      // No space after the colon. The specification makes one optional and
      // tells clients to strip it, so both forms are legal on the wire.
      expect(body, contains('id:1'));
      expect(body, contains('data:{"tick":1}'));
      expect(body, contains('id:5'));
    });

    test('Last-Event-ID resumes rather than replaying', () async {
      // A server that ignores it drops whatever happened while the client was
      // away, which is the reconnect the browser makes automatically.
      final app = await example(sse.buildApp());

      final body =
          (await app.get('/ticks', headers: {'last-event-id': '2'})).body;

      expect(body, contains('id:3'));
      expect(body, isNot(contains('id:1')));
    });

    test('a named event is distinguishable from a default one', () async {
      final app = await example(sse.buildApp());

      final body = (await app.get('/progress')).body;

      expect(body, contains('event:step'));
      expect(body, contains('data:fetching'));
      expect(body, contains('event:done'));
    });
  });

  group('websockets', () {
    /// The headers a real handshake sends.
    Map<String, String> handshake() => const {
          'connection': 'Upgrade',
          'upgrade': 'websocket',
          'sec-websocket-version': '13',
          'sec-websocket-key': 'dGhlIHNhbXBsZSBub25jZQ==',
        };

    test('an upgrade without a ticket is refused while it is still HTTP',
        () async {
      // After the upgrade there is no status code to send, which is why the
      // check belongs here.
      final app = await example(websockets.buildApp());

      final response = await app.raw('GET', '/echo', headers: handshake());

      expect(response.statusCode, 401);
    });

    test(
        'a foreign origin is refused, because same-origin does not apply to '
        'WebSockets', () async {
      // Any page on the internet may open one carrying the user's cookies.
      // Checking Origin is the whole defence.
      final app = await example(websockets.buildApp());

      final response = await app.raw(
        'GET',
        '/echo?token=t-ada',
        headers: {...handshake(), 'origin': 'https://evil.example'},
      );

      expect(response.statusCode, 403);
    });

    test('an allowed origin with a ticket upgrades', () async {
      final app = await example(websockets.buildApp());

      final response = await app.raw(
        'GET',
        '/echo?token=t-ada',
        headers: {...handshake(), 'origin': 'http://localhost:3000'},
      );

      expect(response.statusCode, 101);
    });

    test('a plain GET to an upgrade route is not an upgrade', () async {
      final app = await example(websockets.buildApp());

      expect((await app.get('/echo?token=t-ada')).statusCode, isNot(101));
    });

    test('echo round-trips a message', () async {
      final app = await example(websockets.buildApp());

      final socket = await WebSocket.connect(
        'ws://${app.uri('/echo?token=t-ada').authority}'
        '/echo?token=t-ada',
        headers: {'origin': 'http://localhost:3000'},
      );
      addTearDown(socket.close);

      socket.add('ping');

      expect(await socket.first, 'echo: ping');
    });

    test('the negotiated subprotocol reaches the handler', () async {
      // It used to be dropped, so session.protocol was always null.
      final app = await example(websockets.buildApp());

      final socket = await WebSocket.connect(
        'ws://${app.uri('/greeter?token=t-ada').authority}'
        '/greeter?token=t-ada',
        protocols: const ['greeting.v2'],
        headers: {'origin': 'http://localhost:3000'},
      );
      addTearDown(socket.close);

      expect(socket.protocol, 'greeting.v2');
      expect(await socket.first, '{"hello":true}');
    });
  });
}
