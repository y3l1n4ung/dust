import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../../support.dart';

/// `mount('/')` and the order a route table is read in.
///
/// Both of these were wrong, and both were silent. A root mount claimed only
/// the bare root, so serving a single-page build — the reason `mount('/')`
/// exists — answered 404 for every deep link and every asset. And a router's own
/// routes were flattened ahead of its children whatever the declaration order,
/// so a `mount('/')` written last still shadowed the routes nested above it.

Future<Response> _files(Request request) async => Response.ok('files');

List<String> _notes(Request request) => const ['notes'];

void main() {
  group('a mount at the root', () {
    test('serves the bare root', () async {
      final app = Router()..mount('/', _files);

      final response = await app.handler(request('GET', '/'));

      expect(response.statusCode, 200);
      expect(await response.readAsString(), 'files');
    });

    test('serves a path below it, which is what a deep link is', () async {
      final app = Router()..mount('/', _files);

      final response = await app.handler(request('GET', '/orders/41'));

      expect(response.statusCode, 200);
      expect(await response.readAsString(), 'files');
    });

    test('serves a file beside it', () async {
      final app = Router()..mount('/', _files);

      expect((await app.handler(request('GET', '/main.js'))).statusCode, 200);
    });

    test('still hands the handler the full path', () async {
      // A static handler mounted at the root expects absolute paths: there is
      // no prefix to strip.
      String? seen;
      final app = Router()
        ..mount('/', (request) async {
          seen = request.url.path;
          return Response.ok('ok');
        });

      await app.handler(request('GET', '/css/app.css'));

      expect(seen, 'css/app.css');
    });
  });

  group('declaration order', () {
    test('a route declared before a root mount wins', () async {
      final app = Router()
        ..route('/health', get(_notes))
        ..mount('/', _files);

      final response = await app.handler(request('GET', '/health'));

      expect(await response.readAsString(), '["notes"]');
    });

    test('a nested router declared before a root mount wins', () async {
      // The case that motivated this: an API nested above a static handler
      // serving the front end.
      final app = Router()
        ..nest('/api', Router()..route('/notes', get(_notes)))
        ..mount('/', _files);

      final response = await app.handler(request('GET', '/api/notes'));

      expect(await response.readAsString(), '["notes"]');
    });

    test('a root mount declared first wins, because order is the rule',
        () async {
      final app = Router()
        ..mount('/', _files)
        ..nest('/api', Router()..route('/notes', get(_notes)));

      final response = await app.handler(request('GET', '/api/notes'));

      expect(await response.readAsString(), 'files');
    });

    test('holds across three levels', () async {
      final app = Router()
        ..nest(
          '/a',
          Router()
            ..route('/first', get(_notes))
            ..nest('/b', Router()..route('/second', get(_notes))),
        )
        ..mount('/', _files);

      expect(
        await (await app.handler(request('GET', '/a/first'))).readAsString(),
        '["notes"]',
      );
      expect(
        await (await app.handler(request('GET', '/a/b/second'))).readAsString(),
        '["notes"]',
      );
    });

    test('a prefixed mount claims only its own subtree', () async {
      final app = Router()
        ..mount('/static', _files)
        ..route('/health', get(_notes));

      expect(
        (await app.handler(request('GET', '/static/app.js'))).statusCode,
        200,
      );
      expect(
        await (await app.handler(request('GET', '/health'))).readAsString(),
        '["notes"]',
      );
    });
  });
}
