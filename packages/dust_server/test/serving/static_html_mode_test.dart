import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// Serving a built single-page application has two failure modes that only
/// show up after a deploy: a client-side route answering 404, and a cached
/// shell pinning users to the previous build. Both are cheap to get wrong and
/// invisible in development, so both are pinned here against a directory
/// shaped like real build output — a Flutter one, because it is the shape with
/// the most unhashed entry files to get wrong.

void main() {
  late Directory build;
  late Handler app;

  setUp(() async {
    build = await Directory.systemTemp.createTemp('flutter_build');

    void write(String relative, String contents) {
      File('${build.path}/$relative')
        ..parent.createSync(recursive: true)
        ..writeAsStringSync(contents);
    }

    write('index.html', '<!doctype html><title>app</title>');
    write('main.dart.js', 'console.log(1)');
    write('flutter_service_worker.js', 'self.addEventListener');
    write('version.json', '{"version":"1.0.0"}');
    write('assets/AssetManifest.bin', 'binary');
    write('assets/fonts/MaterialIcons-Regular.otf', 'font');
    write('canvaskit/canvaskit.wasm', 'wasm');

    app = staticFiles(build.path, html: true);
  });

  tearDown(() => build.delete(recursive: true));

  Future<Response> send(String path, {String method = 'GET'}) => Future.value(
        app(Request(method, Uri.parse('http://localhost$path'))),
      ).then((response) => response);

  group('serving the build', () {
    test('answers the document at the root', () async {
      final response = await send('/');

      expect(response.statusCode, 200);
      expect(await response.readAsString(), contains('<title>app</title>'));
    });

    test('serves the compiled application', () async {
      final response = await send('/main.dart.js');

      expect(await response.readAsString(), 'console.log(1)');
    });

    test('serves an asset from a nested directory', () async {
      final response = await send('/assets/fonts/MaterialIcons-Regular.otf');

      expect(response.statusCode, 200);
    });

    test('gives wasm the content type a browser will stream', () async {
      final response = await send('/canvaskit/canvaskit.wasm');

      expect(response.headers['content-type'], 'application/wasm');
    });
  });

  group('client-side routes', () {
    test('reach the application instead of answering 404', () async {
      final response = await send('/orders/42');

      expect(response.statusCode, 200);
      expect(await response.readAsString(), contains('<title>app</title>'));
    });

    test('work several levels deep', () async {
      final response = await send('/a/b/c/d');

      expect(response.statusCode, 200);
    });

    test('work for a path that looks like a missing file', () async {
      final response = await send('/settings.html');

      expect(response.statusCode, 200);
      expect(await response.readAsString(), contains('<title>app</title>'));
    });

    test('keep their query string out of the shell lookup', () async {
      final response = await send('/orders?page=2');

      expect(response.statusCode, 200);
    });

    test('are not served for a POST', () async {
      final response = await send('/orders/42', method: 'POST');

      expect(response.statusCode, 404);
    });

    test('are not served for a PUT', () async {
      final response = await send('/orders/42', method: 'PUT');

      expect(response.statusCode, 404);
    });
  });

  group('caching', () {
    test('revalidates the document, so a deploy reaches users', () async {
      expect((await send('/')).headers['cache-control'], 'no-cache');
    });

    test('revalidates the compiled application', () async {
      expect(
        (await send('/main.dart.js')).headers['cache-control'],
        'no-cache',
      );
    });

    test('revalidates the service worker', () async {
      expect(
        (await send('/flutter_service_worker.js')).headers['cache-control'],
        'no-cache',
      );
    });

    test('revalidates the version file', () async {
      expect(
        (await send('/version.json')).headers['cache-control'],
        'no-cache',
      );
    });

    test('revalidates the shell served for a client-side route', () async {
      expect((await send('/orders/42')).headers['cache-control'], 'no-cache');
    });

    test('caches a hashed asset hard', () async {
      expect(
        (await send('/assets/AssetManifest.bin')).headers['cache-control'],
        'public, max-age=31536000, immutable',
      );
    });

    test('caches the renderer hard', () async {
      expect(
        (await send('/canvaskit/canvaskit.wasm')).headers['cache-control'],
        contains('immutable'),
      );
    });

    test('honours a shorter budget for immutable assets', () async {
      final short = staticFiles(
        build.path,
        html: true,
        immutableFor: const Duration(hours: 1),
      );
      final response = await short(
        Request('GET', Uri.parse('http://localhost/assets/AssetManifest.bin')),
      );

      expect(response.headers['cache-control'], contains('max-age=3600'));
    });
  });

  group('cross-origin isolation', () {
    test('is off unless a wasm build asks for it', () async {
      final response = await send('/');

      expect(response.headers, isNot(contains('cross-origin-opener-policy')));
    });

    test('is applied to every response when asked for', () async {
      final isolated =
          staticFiles(build.path, html: true, crossOriginIsolated: true);

      for (final path in ['/', '/main.dart.js', '/deep/route']) {
        final response = await isolated(
          Request('GET', Uri.parse('http://localhost$path')),
        );

        expect(response.headers['cross-origin-opener-policy'], 'same-origin');
        expect(
            response.headers['cross-origin-embedder-policy'], 'require-corp');
      }
    });
  });

  group('a toolchain that is not Flutter', () {
    test('serves a bundle whose document has another name', () async {
      File('${build.path}/app.html').writeAsStringSync('<title>vite</title>');

      final vite =
          staticFiles(build.path, html: true, defaultDocument: 'app.html');
      final response = await vite(
        Request('GET', Uri.parse('http://localhost/orders/42')),
      );

      expect(await response.readAsString(), contains('<title>vite</title>'));
    });

    test('revalidates the document it was told to use', () async {
      File('${build.path}/app.html').writeAsStringSync('<title>vite</title>');

      final vite = staticFiles(
        build.path,
        html: true,
        defaultDocument: 'app.html',
        revalidate: const {'app.html'},
      );
      final response = await vite(
        Request('GET', Uri.parse('http://localhost/')),
      );

      expect(response.headers['cache-control'], 'no-cache');
    });

    test('caches what the given set does not name', () async {
      // A bundler that hashes every asset needs only the document revalidated,
      // so the Flutter entry files fall into the immutable bucket.
      final hashed =
          staticFiles(build.path, html: true, revalidate: const {'index.html'});
      final response = await hashed(
        Request('GET', Uri.parse('http://localhost/main.dart.js')),
      );

      expect(response.headers['cache-control'], contains('immutable'));
    });
  });

  group('a build that is not there', () {
    test('answers 404 rather than pretending', () async {
      final empty = await Directory.systemTemp.createTemp('empty_build');
      addTearDown(() => empty.delete(recursive: true));

      final response = await staticFiles(empty.path, html: true)(
        Request('GET', Uri.parse('http://localhost/orders/42')),
      );

      expect(response.statusCode, 404);
    });
  });
}
