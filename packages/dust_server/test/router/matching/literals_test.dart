import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('literal segments', () {
    test('a dot matches only itself', () async {
      final app = Router()..route('/a.b', get(label('literal')));
      final handler = app.handler;

      expect((await handler(request('GET', '/a.b'))).statusCode, 200);
      expect((await handler(request('GET', '/axb'))).statusCode, 404);
    });

    test('regex metacharacters are escaped', () async {
      final app = Router()..route(r'/we$ird+(chars)', get(label('literal')));

      expect(
        (await app.handler(request('GET', r'/we$ird+(chars)'))).statusCode,
        200,
      );
    });

    test('matching is case sensitive', () async {
      final app = Router()..route('/Case', get(label('exact')));
      final handler = app.handler;

      expect((await handler(request('GET', '/Case'))).statusCode, 200);
      expect((await handler(request('GET', '/case'))).statusCode, 404);
    });

    test('the root path is servable', () async {
      final app = Router()..route('/', get(label('root')));

      expect(await (await app.handler(request('GET', '/'))).readAsString(),
          'root');
    });
  });
}
