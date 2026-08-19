import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('fallback', () {
    test('answers when nothing matched', () async {
      final app = Router()
        ..route('/a', get(label('a')))
        ..fallback(label('fallback'));

      expect(
        await (await app.handler(request('GET', '/zzz'))).readAsString(),
        'fallback',
      );
    });

    test('does not replace a matched route', () async {
      final app = Router()
        ..route('/a', get(label('a')))
        ..fallback(label('fallback'));

      expect(
          await (await app.handler(request('GET', '/a'))).readAsString(), 'a');
    });

    test('does not replace a 405', () async {
      final app = Router()
        ..route('/a', get(label('a')))
        ..fallback(label('fallback'));

      expect((await app.handler(request('DELETE', '/a'))).statusCode, 405);
    });

    test('leaves the JSON 404 in place when unset', () async {
      final app = Router()..route('/a', get(label('a')));

      final response = await app.handler(request('GET', '/zzz'));

      expect(response.statusCode, 404);
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'no route for /zzz'},
      );
    });
  });
}
