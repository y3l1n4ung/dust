import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// That a static file handler cannot be walked out of.
///
/// It could not be when this was written. The point of the test is that nobody
/// finds out otherwise by accident: path handling is exactly the kind of thing a
/// dependency bump or a "harmless" normalization change breaks silently, and the
/// failure is reading files off the server.

/// Sends [rawPath] byte for byte, without a client normalizing it first.
///
/// `HttpClient` resolves `..` while parsing the URI, so a traversal never
/// reaches the server through it — which would make this test pass for the wrong
/// reason. A socket sends what it is given.
Future<({int status, String body})> sendRaw(int port, String rawPath) async {
  final socket = await Socket.connect('127.0.0.1', port);
  socket.write('GET $rawPath HTTP/1.1\r\nHost: localhost\r\n'
      'Connection: close\r\n\r\n');
  await socket.flush();

  final text = await utf8.decodeStream(socket.cast<List<int>>());
  return (
    status: int.parse(RegExp(r'HTTP/1.1 (\d+)').firstMatch(text)!.group(1)!),
    body: text,
  );
}

void main() {
  group('a static file handler', () {
    late Directory root;
    late File secret;
    late ServerHandle server;

    setUp(() async {
      root = await Directory.systemTemp.createTemp('dust-traversal-');
      await File('${root.path}/index.html').writeAsString('<p>public</p>');
      // One level above what is served. Reaching it is the failure.
      secret = File('${root.parent.path}/dust-test-secret.txt');
      await secret.writeAsString('TOP SECRET');

      server = await serve(
        Router()..mount('/', staticFiles(root.path)),
        InternetAddress.loopbackIPv4,
        0,
      );
    });

    tearDown(() async {
      await server.close(drain: const Duration(seconds: 1));
      if (secret.existsSync()) await secret.delete();
      if (root.existsSync()) await root.delete(recursive: true);
    });

    test('serves what is inside it', () async {
      final response = await sendRaw(server.port, '/index.html');

      expect(response.status, 200);
      expect(response.body, contains('public'));
    });

    test('refuses every shape of traversal', () async {
      // Plain, percent-encoded, mixed, double-encoded, and the doubled-dot
      // trick that defeats a single naive replace.
      const attempts = [
        '/../dust-test-secret.txt',
        '/%2e%2e%2fdust-test-secret.txt',
        '/%2e%2e/dust-test-secret.txt',
        '/..%2fdust-test-secret.txt',
        '/%252e%252e%252fdust-test-secret.txt',
        '/....//dust-test-secret.txt',
        '/./../dust-test-secret.txt',
        r'/..\dust-test-secret.txt',
      ];

      for (final attempt in attempts) {
        final response = await sendRaw(server.port, attempt);

        expect(
          response.body,
          isNot(contains('TOP SECRET')),
          reason: 'traversal succeeded for $attempt',
        );
        expect(response.status, isNot(200), reason: attempt);
      }
    });

    test('a single-page fallback does not become a traversal', () async {
      // With `html: true` an unknown path answers the document. That must be
      // the document, and never a file from outside the directory.
      final spa = await serve(
        Router()..mount('/', staticFiles(root.path, html: true)),
        InternetAddress.loopbackIPv4,
        0,
      );
      addTearDown(() => spa.close(drain: const Duration(seconds: 1)));

      final response =
          await sendRaw(spa.port, '/%2e%2e%2fdust-test-secret.txt');

      expect(response.status, 200);
      expect(response.body, contains('public'));
      expect(response.body, isNot(contains('TOP SECRET')));
    });
  });
}
