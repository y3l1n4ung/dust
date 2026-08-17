import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

/// `PeerExtractable` reads what `shelf_io` puts in the context, and what it
/// puts there is an `HttpConnectionInfo` that only a real connection produces.
/// A constructed request cannot reach this branch, so it is tested over a
/// loopback socket.

void main() {
  late ServerHandle server;

  setUp(() async {
    final app = Router()
      ..route('/whoami', get((request) async {
        final info = await peer().require(request);
        return {
          'remoteAddress': info.remoteAddress,
          'remotePort': info.remotePort,
          'localPort': info.localPort,
        };
      }))
      ..route('/described', get((request) async {
        final info = await peer().require(request);
        return {'text': info.toString()};
      }));

    server = await serveRouter(app, InternetAddress.loopbackIPv4, 0);
  });

  tearDown(() => server.close());

  Future<Map<String, Object?>> fetch(String path) async {
    final response = await http.get(
      Uri.parse('http://${server.address.host}:${server.port}$path'),
    );
    return (jsonDecode(response.body) as Map).cast<String, Object?>();
  }

  group('the peer of a real connection', () {
    test('reports the loopback address the client came from', () async {
      final body = await fetch('/whoami');

      expect(body['remoteAddress'], InternetAddress.loopbackIPv4.address);
    });

    test('reports the port the server is listening on', () async {
      final body = await fetch('/whoami');

      expect(body['localPort'], server.port);
    });

    test('reports a remote port the client actually used', () async {
      final body = await fetch('/whoami');

      expect(body['remotePort'], isA<int>());
      expect(body['remotePort'], greaterThan(0));
    });

    test('gives each connection its own remote port', () async {
      final first = await fetch('/whoami');
      final second = await fetch('/whoami');

      expect(first['localPort'], second['localPort']);
      expect(first['remoteAddress'], second['remoteAddress']);
    });

    test('describes itself as address and port', () async {
      final body = await fetch('/described');

      expect(
        body['text'],
        startsWith('PeerInfo(${InternetAddress.loopbackIPv4.address}:'),
      );
    });
  });
}
