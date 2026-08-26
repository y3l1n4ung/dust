import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';
import 'package:web_socket_channel/io.dart';

/// Layers wrap every route, including the ones that upgrade. An upgrade
/// succeeds by throwing `HijackException` to take over the socket, so a layer
/// that treats it as a failure — or that tries to decorate the response that
/// never comes — breaks WebSockets while every HTTP test still passes.
///
/// This is the combination the two examples miss: one has layers and no
/// sockets, the other has sockets and no timeout.

void main() {
  late ServerHandle server;
  late List<Object> errors;
  late List<AccessRecord> records;
  late List<Request> timedOut;

  setUp(() async {
    errors = [];
    records = [];
    timedOut = [];

    final app = Router(onError: (error, stack) => errors.add(error))
      ..layer(RequestTimeout(
        const Duration(milliseconds: 300),
        onTimeout: timedOut.add,
      ))
      ..layer(const RequestId())
      ..layer(AccessLog(records.add))
      ..route('/echo', ws((session) async {
        await for (final message in session.textMessages) {
          session.send('echo:$message');
        }
      }))
      ..route('/quick', get((request) async => {'ok': true}));

    server = await serve(app, InternetAddress.loopbackIPv4, 0);
  });

  tearDown(() => server.close(drain: const Duration(seconds: 2)));

  String origin() => '${server.address.host}:${server.port}';

  WebSocketChannel connect() =>
      IOWebSocketChannel.connect(Uri.parse('ws://${origin()}/echo'));

  group('an upgrade under a timeout layer', () {
    test('completes rather than answering 503', () async {
      final socket = connect();
      socket.sink.add('hello');

      expect(await socket.stream.first, 'echo:hello');

      await socket.sink.close();
    });

    test('stays open past the budget', () async {
      final socket = connect();
      final received = socket.stream.take(2).cast<String>().toList();

      socket.sink.add('first');
      // The budget is 300ms; a conversation outlives any one request.
      await Future<void>.delayed(const Duration(milliseconds: 600));
      socket.sink.add('second');

      expect(await received, ['echo:first', 'echo:second']);

      await socket.sink.close();
    });

    test('is not recorded as having timed out', () async {
      final socket = connect();
      socket.sink.add('hello');
      await socket.stream.first;
      await Future<void>.delayed(const Duration(milliseconds: 500));

      expect(timedOut, isEmpty);

      await socket.sink.close();
    });

    test('does not report the hijack as a handler failure', () async {
      final socket = connect();
      socket.sink.add('hello');
      await socket.stream.first;

      expect(errors, isEmpty);

      await socket.sink.close();
    });
  });

  group('the other layers', () {
    test('still answer an ordinary request beside the upgrade', () async {
      final socket = connect();
      socket.sink.add('hello');
      await socket.stream.first;

      final response = await HttpClient()
          .getUrl(Uri.parse('http://${origin()}/quick'))
          .then((request) => request.close());

      expect(response.statusCode, 200);
      expect(response.headers.value('x-request-id'), isNotNull);

      await socket.sink.close();
    });

    test('record the ordinary request in the access log', () async {
      await HttpClient()
          .getUrl(Uri.parse('http://${origin()}/quick'))
          .then((request) => request.close());

      expect(records.map((record) => record.path), contains('/quick'));
    });
  });
}
