import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';
import 'package:web_socket_channel/io.dart';

/// The parts of a session beyond text echo: binary frames, the negotiated
/// subprotocol, and what the peer's close looks like from the server side.
/// All of it over a real socket, because a framing mistake only shows up on
/// the wire.

void main() {
  late ServerHandle server;
  late List<Object> errors;

  /// What each handler observed, read back over HTTP so a test can assert on
  /// server-side state without reaching into the handler.
  late Map<String, Object?> observed;

  setUp(() async {
    errors = [];
    observed = {};

    final app = Router(onError: (error, stack) => errors.add(error))
      ..route('/binary', ws((session) async {
        await for (final bytes in session.binaryMessages) {
          session.sendBytes(Uint8List.fromList(bytes.reversed.toList()));
        }
      }))
      ..route('/mixed', ws((session) async {
        final seen = <String>[];
        await for (final message in session.messages) {
          seen.add(message is String ? 'text' : 'binary');
          if (seen.length == 2) {
            observed['kinds'] = seen;
            session.send(seen.join(','));
            return;
          }
        }
      }))
      ..route(
          '/protocol',
          ws(protocols: const ['chat.v2', 'chat.v1'], (session) async {
            session.send('protocol:${session.protocol}');
            await session.close();
          }))
      ..route('/farewell', ws((session) async {
        // Draining is what lets the close frame be seen; `done` completes once
        // the stream has ended.
        await session.messages.drain<void>();
        await session.done;
        observed['code'] = session.closeCode;
        observed['reason'] = session.closeReason;
      }))
      ..route('/observed', get((request) async => observed));

    server = await serveRouter(app, InternetAddress.loopbackIPv4, 0);
  });

  tearDown(() => server.close());

  String origin() => '${server.address.host}:${server.port}';

  WebSocketChannel connect(String path, {Iterable<String>? protocols}) =>
      IOWebSocketChannel.connect(
        Uri.parse('ws://${origin()}$path'),
        protocols: protocols,
      );

  group('binary frames', () {
    test('arrive on binaryMessages and go back out with sendBytes', () async {
      final socket = connect('/binary');
      socket.sink.add([1, 2, 3]);

      expect(await socket.stream.first, [3, 2, 1]);

      await socket.sink.close();
    });

    test('are typed as bytes rather than text', () async {
      final socket = connect('/binary');
      socket.sink.add(Uint8List.fromList([7, 8]));

      expect(await socket.stream.first, isA<List<int>>());

      await socket.sink.close();
    });
  });

  group('messages', () {
    test('carries text and binary on one stream', () async {
      final socket = connect('/mixed');
      socket.sink
        ..add('first')
        ..add([1, 2]);

      expect(await socket.stream.first, 'text,binary');

      await socket.sink.close();
    });
  });

  group('protocol', () {
    test('reports the subprotocol the handshake settled on', () async {
      final socket = connect('/protocol', protocols: const ['chat.v1']);

      expect(await socket.stream.first, 'protocol:chat.v1');
    });

    test('picks the one both sides offer', () async {
      final socket = connect(
        '/protocol',
        protocols: const ['chat.v9', 'chat.v2'],
      );

      expect(await socket.stream.first, 'protocol:chat.v2');
    });
  });

  group('the peer closing', () {
    test('completes done and reports the code and reason', () async {
      final socket = connect('/farewell');
      await socket.sink.close(4321, 'goodbye');

      // The handler writes what it saw once `done` completes; poll the plain
      // HTTP route on the same router until it has.
      var seen = const <String, Object?>{};
      for (var attempt = 0; attempt < 50; attempt++) {
        final response = await HttpClient()
            .getUrl(Uri.parse('http://${origin()}/observed'))
            .then((request) => request.close());
        final body = await response.transform(const Utf8Decoder()).join();
        seen = (jsonDecode(body) as Map).cast<String, Object?>();
        if (seen.isNotEmpty) break;
        await Future<void>.delayed(const Duration(milliseconds: 20));
      }

      expect(seen, {'code': 4321, 'reason': 'goodbye'});
      expect(errors, isEmpty);
    });
  });
}
