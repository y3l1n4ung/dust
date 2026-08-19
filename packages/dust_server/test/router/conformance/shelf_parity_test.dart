import 'package:dust_server/server.dart';
import 'package:shelf_router/shelf_router.dart' as shelf_router;
import 'package:test/test.dart';

import '../support.dart';

/// Checks our matcher against `shelf_router`, the package the ecosystem
/// already trusts, on the same route table.
///
/// Where the two disagree it should be on purpose, and every intentional
/// difference is asserted at the bottom of this file rather than left to be
/// discovered later.

const _routes = <String>[
  '/',
  '/health',
  '/todos',
  '/todos/{id}',
  '/todos/{id}/notes',
  '/todos/{id}/notes/{noteId}',
  '/files/{name}',
  '/a/b/c',
  '/a.b',
];

const _requests = <String>[
  '/',
  '/health',
  '/health/',
  '/healthz',
  '/todos',
  '/todos/',
  '/todos/7',
  '/todos/7/',
  '/todos/7/notes',
  '/todos/7/notes/9',
  '/todos/7/notes/9/extra',
  '/todos/a%20b',
  '/todos/a%2Fb',
  '/todos/caf%C3%A9',
  '/files/report.pdf',
  '/files/report.pdf/extra',
  '/a/b/c',
  '/a/b',
  '/a.b',
  '/axb',
  '/A/B/C',
  '/todos//7',
  '/unknown',
];

Handler _label(String route) => (request) async => textResponse(route);

String _toShelfPath(String path) => path.replaceAllMapped(
      RegExp(r'\{([^/}]+)\}'),
      (match) => '<${match.group(1)}>',
    );

Future<String> _ours(String path) async {
  final app = Router();
  for (final route in _routes) {
    app.route(route, get(_label(route)));
  }

  final response = await app.handler(request('GET', path));
  return response.statusCode == 200
      ? await response.readAsString()
      : '${response.statusCode}';
}

Future<String> _theirs(String path) async {
  final router = shelf_router.Router();
  for (final route in _routes) {
    router.get(_toShelfPath(route), _label(route));
  }

  final response = await router.call(request('GET', path));
  return response.statusCode == 200
      ? await response.readAsString()
      : '${response.statusCode}';
}

void main() {
  group('agrees with shelf_router', () {
    for (final path in _requests) {
      test('on $path', () async {
        expect(await _ours(path), await _theirs(path));
      });
    }
  });

  group('differs from shelf_router on purpose', () {
    test('answers a known path with the wrong method as 405, not 404',
        () async {
      final app = Router()..route('/todos', get(_label('list')));
      final router = shelf_router.Router()..get('/todos', _label('list'));

      expect(
        (await app.handler(request('DELETE', '/todos'))).statusCode,
        405,
      );
      expect((await router.call(request('DELETE', '/todos'))).statusCode, 404);
    });

    test('answers an unknown path with a JSON body', () async {
      final app = Router()..route('/todos', get(_label('list')));

      final response = await app.handler(request('GET', '/nope'));

      expect(response.headers['content-type'], 'application/json');
      expect(await response.readAsString(), contains('no route for'));
    });
  });
}
