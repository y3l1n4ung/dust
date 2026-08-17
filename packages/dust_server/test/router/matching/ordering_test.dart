import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Declaration order decides which of two matching routes runs. The matcher
/// buckets routes by first segment for speed, so these pin that the bucketing
/// did not quietly become a specificity rule.

void main() {
  group('declaration order', () {
    test('a literal declared first beats a later parameter', () async {
      final app = Router()
        ..route('/a/fixed', get(label('literal')))
        ..route('/a/{id}', get(label('parameter')));

      expect(
        await (await app.handler(request('GET', '/a/fixed'))).readAsString(),
        'literal',
      );
    });

    test('a parameter declared first beats a later literal', () async {
      final app = Router()
        ..route('/a/{id}', get(label('parameter')))
        ..route('/a/fixed', get(label('literal')));

      expect(
        await (await app.handler(request('GET', '/a/fixed'))).readAsString(),
        'parameter',
      );
    });

    test('holds across buckets, literal first', () async {
      final app = Router()
        ..route('/a', get(label('literal')))
        ..route('/{anything}', get(label('parameter')));

      expect(
        await (await app.handler(request('GET', '/a'))).readAsString(),
        'literal',
      );
    });

    test('holds across buckets, parameter first', () async {
      final app = Router()
        ..route('/{anything}', get(label('parameter')))
        ..route('/a', get(label('literal')));

      expect(
        await (await app.handler(request('GET', '/a'))).readAsString(),
        'parameter',
      );
    });

    test('holds for a mount against a later route', () async {
      final app = Router()
        ..mount('/m', label('mount'))
        ..route('/m/inner', get(label('route')));

      expect(
        await (await app.handler(request('GET', '/m/inner'))).readAsString(),
        'mount',
      );
    });

    test('holds for a route against a later mount', () async {
      final app = Router()
        ..route('/m/inner', get(label('route')))
        ..mount('/m', label('mount'));

      expect(
        await (await app.handler(request('GET', '/m/inner'))).readAsString(),
        'route',
      );
    });

    test('holds for a catch-all against an earlier exact route', () async {
      final app = Router()
        ..route('/files/readme', get(label('exact')))
        ..route('/files/{*rest}', get(label('catch-all')));

      expect(
        await (await app.handler(request('GET', '/files/readme')))
            .readAsString(),
        'exact',
      );
    });
  });
}
