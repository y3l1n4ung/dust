import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('any', () {
    test('serves a method nothing else claims', () async {
      final app = Router()..route('/a', any(label('any')));
      final handler = app.handler;

      expect(await (await handler(request('GET', '/a'))).readAsString(), 'any');
      expect(
        await (await handler(request('PROPFIND', '/a'))).readAsString(),
        'any',
      );
    });

    test('loses to an explicit method on the same path', () async {
      final app = Router()..route('/a', get(label('get')).any(label('rest')));
      final handler = app.handler;

      expect(await (await handler(request('GET', '/a'))).readAsString(), 'get');
      expect(
        await (await handler(request('DELETE', '/a'))).readAsString(),
        'rest',
      );
    });

    test('answers instead of a 405', () async {
      final app = Router()..route('/a', any(label('any')));

      expect((await app.handler(request('DELETE', '/a'))).statusCode, 200);
    });

    test('does not answer for another path', () async {
      final app = Router()..route('/a', any(label('any')));

      expect((await app.handler(request('GET', '/b'))).statusCode, 404);
    });

    test('is registered under the any-method marker', () {
      expect(any(label('x')).handlers.keys, [Route.anyMethod]);
    });
  });
}
