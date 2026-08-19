import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('405', () {
    test('is answered for a known path with the wrong method', () async {
      final response = await sampleRouter().handler(request('DELETE', '/a'));

      expect(response.statusCode, 405);
    });

    test('carries a sorted Allow header', () async {
      final app = Router()
        ..route('/a', post(label('p')).delete(label('d')).put(label('u')));

      expect(
        (await app.handler(request('PATCH', '/a'))).headers['allow'],
        'DELETE, POST, PUT',
      );
    });

    test('names the method and path in a JSON body', () async {
      final response = await sampleRouter().handler(request('DELETE', '/a'));

      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'DELETE is not allowed on /a'},
      );
    });

    test('is not answered for an unknown path', () async {
      final response = await sampleRouter().handler(request('DELETE', '/zzz'));

      expect(response.statusCode, 404);
      expect(response.headers.containsKey('allow'), isFalse);
    });

    test('considers only the path that matched', () async {
      final app = Router()
        ..route('/a', get(label('a')))
        ..route('/b', post(label('b')));

      expect((await app.handler(request('POST', '/a'))).headers['allow'],
          'GET, HEAD');
    });
  });
}
