import 'dart:io';
import 'package:test/test.dart';
import '../../example/global_404.dart' as global_404;
import '../../example/handle_head_request.dart' as head_request;
import '../../example/static_files.dart' as static_files;
import '../../example/templates.dart' as templates;
import '../../example/versioning.dart' as versioning;
import 'serve.dart';

/// What a handler sends back, including streams and sockets.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
/// Templates, static files, fallbacks and versioning.
void main() {
  group('templates', () {
    test('renders a page inside the shared layout', () async {
      final app = await example(templates.buildApp());

      final response = await app.get('/');

      expect(response.headers['content-type'], 'text/html; charset=utf-8');
      expect(response.body, startsWith('<!doctype html>'));
      expect('<head>'.allMatches(response.body).length, 1);
      expect(response.body, contains('href="/notes/1"'));
    });

    test('interpolation is escaped, which is the reason to use an engine',
        () async {
      final app = await example(templates.buildApp());

      final response = await app.get('/notes/2');

      expect(response.body, contains('&lt;script&gt;'));
      expect(response.body, isNot(contains('<script>alert(1)</script>')));
    });

    test('an unknown id answers 404 as a page, not as JSON', () async {
      final app = await example(templates.buildApp());

      final response = await app.get('/notes/9');

      expect(response.statusCode, 404);
      expect(response.headers['content-type'], 'text/html; charset=utf-8');
      expect(response.body, contains('Back'));
    });
  });

  group('static_files', () {
    /// Writes a throwaway build for one test.
    Future<String> build() async {
      final root =
          await Directory.systemTemp.createTemp('dust-example-static-');
      addTearDown(() => root.delete(recursive: true));
      await File('${root.path}/index.html').writeAsString(
        '<!doctype html><title>App</title><div id="app">loading</div>',
      );
      await File('${root.path}/main.a1b2c3.js')
          .writeAsString('console.log("fingerprinted");');
      return root.path;
    }

    test('the root serves the default document', () async {
      final app = await example(static_files.buildApp(await build()));

      final response = await app.get('/');

      expect(response.statusCode, 200);
      expect(response.body, contains('<div id="app">'));
    });

    test('a deep link serves the same document, so the client router runs',
        () async {
      // Without html: true this is a 404 — there is no such file, and the
      // router that would have handled it has not loaded yet.
      final app = await example(static_files.buildApp(await build()));

      final response = await app.get('/orders/41');

      expect(response.statusCode, 200);
      expect(response.body, contains('<div id="app">'));
    });

    test('the document is revalidated, not cached hard', () async {
      // It is how a browser learns the new asset names. Cache it for a year and
      // users stay on a deploy you have replaced.
      final app = await example(static_files.buildApp(await build()));

      final cache = (await app.get('/')).headers['cache-control'] ?? '';

      expect(cache, isNot(contains('immutable')));
    });

    test('a fingerprinted asset is immutable', () async {
      final app = await example(static_files.buildApp(await build()));

      final cache =
          (await app.get('/main.a1b2c3.js')).headers['cache-control'] ?? '';

      expect(cache, contains('immutable'));
    });

    test('the API is reachable, because it is mounted first', () async {
      // mount('/') claims everything below it, so order decides whether /api
      // reaches its routes or gets the document.
      final app = await example(static_files.buildApp(await build()));

      expect(app.array(await app.get('/api/notes')), ['first']);
    });
  });

  group('global_404', () {
    test('a browser gets a page', () async {
      final app = await example(global_404.buildApp());

      final response = await app.get('/nothing');

      expect(response.statusCode, 404);
      expect(response.headers['content-type'], 'text/html; charset=utf-8');
    });

    test('an API path gets JSON whatever it says it accepts', () async {
      final app = await example(global_404.buildApp());

      final response = await app.get(
        '/api/nothing',
        headers: {'accept': 'text/html'},
      );

      expect(response.statusCode, 404);
      expect(app.object(response)['error'], 'no such route');
    });

    test('an Accept header alone is enough to ask for JSON', () async {
      final app = await example(global_404.buildApp());

      final response = await app.get(
        '/nothing',
        headers: {'accept': 'application/json'},
      );

      expect(app.object(response)['error'], 'no such route');
    });

    test('a 405 does not reach the fallback', () async {
      // A path that exists for another method is answered with Allow, which is
      // more useful to a client than a 404.
      final app = await example(global_404.buildApp());

      final response = await app.send('PUT', '/api/notes');

      expect(response.statusCode, 405);
      expect(response.headers['allow'], contains('GET'));
    });
  });

  group('handle_head_request', () {
    test('HEAD is answered from the GET route, with no body', () async {
      final app = await example(head_request.buildApp());

      final get = await app.get('/notes');
      final head = await app.send('HEAD', '/notes');

      expect(head.statusCode, 200);
      expect(head.body, isEmpty);
      expect(head.headers['content-type'], get.headers['content-type']);
    });

    test('HEAD appears in Allow', () async {
      final app = await example(head_request.buildApp());

      expect(
          (await app.send('PUT', '/notes')).headers['allow'], contains('HEAD'));
    });

    test('an explicit HEAD route overrides the automatic one', () async {
      final app = await example(head_request.buildApp());

      final head = await app.send('HEAD', '/report');

      expect(head.statusCode, 200);
      expect(head.headers['content-length'], '28');
    });

    test('the handler still runs for HEAD, body discarded', () async {
      // A GET that increments a counter does so for every HEAD too. If that is
      // unwanted, the work does not belong in a GET.
      final app = await example(head_request.buildApp());
      head_request.counter.calls = 0;

      await app.get('/counted');
      await app.send('HEAD', '/counted');

      expect(head_request.counter.calls, 2);
    });
  });

  group('versioning', () {
    test('each path version keeps its own shape', () async {
      final app = await example(versioning.buildApp());

      expect(app.array(await app.get('/v1/notes')), ['first', 'second']);
      expect(
        (app.array(await app.get('/v2/notes')).first! as Map)['id'],
        1,
      );
    });

    test('no version header means the oldest, not the newest', () async {
      // Defaulting to latest breaks a client the day you ship v3 — silently,
      // with no deploy of theirs to blame.
      final app = await example(versioning.buildApp());

      expect(app.array(await app.get('/notes')), ['first', 'second']);
    });

    test('the header selects a version', () async {
      final app = await example(versioning.buildApp());

      final response = await app.get(
        '/notes',
        headers: {'accept': 'application/vnd.notes.v2+json'},
      );

      expect((app.array(response).first! as Map)['title'], 'first');
    });

    test('an unknown version is a 406 naming what exists', () async {
      final app = await example(versioning.buildApp());

      final response = await app.get(
        '/notes',
        headers: {'accept': 'application/vnd.notes.v9+json'},
      );

      expect(response.statusCode, 406);
      expect(app.object(response)['error'], contains('try v1 or v2'));
    });
  });
}
