import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('HEAD', () {
    test('falls back to the GET route', () async {
      final app = Router()..route('/a', get(label('a')));

      expect((await app.handler(request('HEAD', '/a'))).statusCode, 200);
    });

    test('prefers an explicit HEAD route', () async {
      final app = Router()..route('/a', get(label('get')).head(label('head')));

      expect(
        await (await app.handler(request('HEAD', '/a'))).readAsString(),
        'head',
      );
    });

    test('does not fall back when there is no GET', () async {
      final app = Router()..route('/a', post(label('post')));

      final response = await app.handler(request('HEAD', '/a'));

      expect(response.statusCode, 405);
      expect(response.headers['allow'], 'POST');
    });

    test('is listed in Allow wherever GET is', () async {
      final app = Router()..route('/a', get(label('a')).post(label('b')));

      final response = await app.handler(request('DELETE', '/a'));

      expect(response.headers['allow'], 'GET, HEAD, POST');
    });
  });
}
