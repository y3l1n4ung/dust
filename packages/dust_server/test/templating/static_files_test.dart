import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

void main() {
  late Directory root;
  late ServerHandle server;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('dust_static');
    File('${root.path}/index.html').writeAsStringSync('<h1>home</h1>');
    File('${root.path}/app.css').writeAsStringSync('body{color:red}');
    Directory('${root.path}/deep').createSync();
    File('${root.path}/deep/note.txt').writeAsStringSync('nested');
    File('${root.path}/spa.html').writeAsStringSync('<div id="app"></div>');

    final app = Router()
      ..route('/api/ping', get((request) async => jsonResponse({'ok': true})))
      ..mount('/assets', staticFiles(root.path))
      ..fallback(singlePageApp('${root.path}/spa.html'));

    server = await serve(app, InternetAddress.loopbackIPv4, 0);
  });

  tearDown(() async {
    await server.close();
    await root.delete(recursive: true);
  });

  Future<http.Response> fetch(String path) =>
      http.get(Uri.parse('http://${server.address.host}:${server.port}$path'));

  group('static files', () {
    test('serves a file with its content type', () async {
      final response = await fetch('/assets/app.css');

      expect(response.statusCode, 200);
      expect(response.body, 'body{color:red}');
      expect(response.headers['content-type'], contains('text/css'));
    });

    test('serves the default document for the directory', () async {
      final response = await fetch('/assets/');

      expect(response.body, '<h1>home</h1>');
    });

    test('serves a nested file', () async {
      expect((await fetch('/assets/deep/note.txt')).body, 'nested');
    });

    test('answers a missing file within the mount', () async {
      expect((await fetch('/assets/nope.png')).statusCode, 404);
    });

    test('never serves a file outside the directory', () async {
      // Dart normalizes `..` while parsing, so the request never arrives as a
      // traversal: it becomes `/etc/hosts`, matches no route, and falls through
      // to the single-page document. What matters is that no file from outside
      // the directory is ever in the body.
      for (final path in [
        '/assets/../../etc/hosts',
        '/assets/%2e%2e/%2e%2e/etc/hosts',
        '/assets/..%2f..%2fetc%2fhosts',
      ]) {
        final response = await fetch(path);

        expect(response.body, isNot(contains('localhost')));
        expect(response.body, isNot(contains('root:')));
      }
    });

    test('refuses an encoded escape that reaches the handler', () async {
      final response = await fetch('/assets/%2e%2e%2fapp.css');

      expect(response.statusCode, isNot(200));
    });

    test('leaves API routes alone', () async {
      final response = await fetch('/api/ping');

      expect(response.headers['content-type'], 'application/json');
      expect(response.body, '{"ok":true}');
    });
  });

  group('single-page fallback', () {
    test('answers an unknown path with the document', () async {
      final response = await fetch('/todos/42/edit');

      expect(response.statusCode, 200);
      expect(response.body, '<div id="app"></div>');
    });

    test('does not shadow a real route', () async {
      expect((await fetch('/api/ping')).body, '{"ok":true}');
    });

    test('does not shadow a served asset', () async {
      expect((await fetch('/assets/app.css')).body, 'body{color:red}');
    });

    test('answers a nested unknown path with the document', () async {
      expect((await fetch('/a/b/c')).body, '<div id="app"></div>');
    });

    test('does not answer a POST with the document', () async {
      final response = await http.post(
        Uri.parse('http://${server.address.host}:${server.port}/todos'),
      );

      expect(response.statusCode, 404);
      expect(response.body, isNot(contains('<div id="app">')));
    });
  });
}
