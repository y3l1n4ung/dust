import 'dart:math';

import 'package:dust_server/server.dart';
import 'package:shelf_router/shelf_router.dart' as shelf_router;
import 'package:test/test.dart';

import '../support.dart';

/// Adversarial and pathological input. The route table is written by the
/// application author, but the request path, method, and headers are not.

Handler _label(String label) => (request) async => textResponse(label);

String _toShelfPath(String path) => path.replaceAllMapped(
      RegExp(r'\{([^/}]+)\}'),
      (match) => '<${match.group(1)}>',
    );

const _routes = <String>[
  '/',
  '/a',
  '/a/{id}',
  '/a/{id}/b',
  '/a/{id}/b/{other}',
  '/x/y/z',
];

Router _ours() {
  final app = Router();
  for (final route in _routes) {
    app.route(route, get(_label(route)));
  }
  return app;
}

shelf_router.Router _theirs() {
  final router = shelf_router.Router();
  for (final route in _routes) {
    router.get(_toShelfPath(route), _label(route));
  }
  return router;
}

String _randomPath(Random random) {
  const alphabet = ['a', 'b', 'x', 'y', 'z', 'id', '7', '', '.', '%20', '%2F'];
  final segments = random.nextInt(5);
  return '/${[
    for (var i = 0; i < segments; i++)
      alphabet[random.nextInt(alphabet.length)],
  ].join('/')}';
}

void main() {
  group('randomized agreement with shelf_router', () {
    test('1000 generated paths route the same way', () async {
      final random = Random(20260816);
      final ours = _ours().handler;
      final theirs = _theirs();
      final divergences = <String>[];

      for (var i = 0; i < 1000; i++) {
        final path = _randomPath(random);
        final a = await ours(request('GET', path));
        final b = await theirs.call(request('GET', path));

        final mine =
            a.statusCode == 200 ? await a.readAsString() : '${a.statusCode}';
        final yours =
            b.statusCode == 200 ? await b.readAsString() : '${b.statusCode}';

        if (mine != yours) divergences.add('$path: ours=$mine theirs=$yours');
      }

      expect(divergences, isEmpty);
    });
  });
}
