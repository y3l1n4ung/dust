import 'dart:async';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';
import 'package:web_socket_channel/io.dart';

/// WebSockets are served by the same router as HTTP, so these run over a real
/// socket: a handshake that never leaves the process proves nothing.

final class _Hub {
  final rooms = <String, List<WebSocketSession>>{};

  void join(String room, WebSocketSession session) =>
      rooms.putIfAbsent(room, () => []).add(session);

  void broadcast(String room, String message) {
    for (final session in rooms[room] ?? const <WebSocketSession>[]) {
      session.send(message);
    }
  }
}

Future<void> _echo(WebSocketSession session) async {
  await for (final message in session.textMessages) {
    if (message == 'bye') {
      await session.close(1000, 'asked to leave');
      return;
    }
    session.send('echo:$message');
  }
}

void main() {
  late ServerHandle server;
  late _Hub hub;
  final errors = <Object>[];

  setUp(() async {
    hub = _Hub();
    errors.clear();

    final app = Router(onError: (error, stack) => errors.add(error))
      ..route('/echo', ws(_echo))
      ..route('/rooms/{room}', ws((session) async {
        final room = session.pathParameters['room']!;
        hub.join(room, session);
        session.send('joined:$room');
        await for (final message in session.textMessages) {
          hub.broadcast(room, message);
        }
      }))
      ..route('/state', ws((session) async {
        final repo =
            await const StateExtractable<String>().extract(session.request);
        session.send(switch (repo) {
          Ok(:final value) => 'state:$value',
          Err(:final error) => 'error:${error.status}',
        });
        await session.close();
      }))
      ..route('/boom', ws((session) async => throw StateError('boom')))
      ..route('/http', get((request) async => textResponse('plain')))
      ..withState('shared');

    server = await serveRouter(app, InternetAddress.loopbackIPv4, 0);
  });

  tearDown(() => server.close());

  WebSocketChannel connect(String path) => IOWebSocketChannel.connect(
        Uri.parse('ws://${server.address.host}:${server.port}$path'),
      );

  group('a WebSocket route', () {
    test('completes the handshake and echoes', () async {
      final socket = connect('/echo');
      socket.sink.add('hello');

      expect(await socket.stream.first, 'echo:hello');

      await socket.sink.close();
    });

    test('carries several messages on one connection', () async {
      final socket = connect('/echo');
      final received = socket.stream.take(3).cast<String>().toList();

      socket.sink
        ..add('a')
        ..add('b')
        ..add('c');

      expect(await received, ['echo:a', 'echo:b', 'echo:c']);
      await socket.sink.close();
    });

    test('closes with the code the handler chose', () async {
      final socket = connect('/echo');
      socket.sink.add('bye');
      await socket.stream.drain<void>();

      expect(socket.closeCode, 1000);
    });

    test('sees path parameters from the route that matched', () async {
      final socket = connect('/rooms/general');

      expect(await socket.stream.first, 'joined:general');
      await socket.sink.close();
    });

    test('reaches state attached to the router', () async {
      final socket = connect('/state');

      expect(await socket.stream.first, 'state:shared');
      await socket.sink.close();
    });

    test('closes with a failure code when the handler throws', () async {
      final socket = connect('/boom');
      await socket.stream.drain<void>();

      expect(socket.closeCode, WebSocketClose.handlerFailed);
      expect(socket.closeReason, 'handler failed');
      expect(errors.single, isA<StateError>());
    });

    test('serves HTTP routes from the same router', () async {
      final socket = connect('/echo');
      socket.sink.add('still here');

      expect(await socket.stream.first, 'echo:still here');
      await socket.sink.close();
    });

    test('broadcasts between two connections', () async {
      final first = connect('/rooms/lobby');
      final second = connect('/rooms/lobby');

      expect(await first.stream.first, 'joined:lobby');
      final heard = second.stream.skip(1).first;

      first.sink.add('hello room');
      expect(await heard, 'hello room');

      await first.sink.close();
      await second.sink.close();
    });

    test('refuses a plain GET on a WebSocket path', () async {
      // shelf_web_socket answers a request that is not an upgrade with 404
      // rather than 400, and the route does exist, so this pins what a client
      // actually sees.
      final response = await HttpClient()
          .getUrl(
              Uri.parse('http://${server.address.host}:${server.port}/echo'))
          .then((request) => request.close());

      expect(response.statusCode, 404);
    });
  });
}
