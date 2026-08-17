import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('verb builders', () {
    test('cover every method the annotations declare', () {
      expect(get(label('x')).handlers.keys, ['GET']);
      expect(post(label('x')).handlers.keys, ['POST']);
      expect(put(label('x')).handlers.keys, ['PUT']);
      expect(patch(label('x')).handlers.keys, ['PATCH']);
      expect(delete(label('x')).handlers.keys, ['DELETE']);
      expect(head(label('x')).handlers.keys, ['HEAD']);
      expect(options(label('x')).handlers.keys, ['OPTIONS']);
    });

    test('chain onto one path', () {
      expect(
        get(label('a')).post(label('b')).delete(label('c')).handlers.keys,
        ['GET', 'POST', 'DELETE'],
      );
    });

    test('refuse the same method twice', () {
      expect(() => get(label('a')).get(label('b')), throwsArgumentError);
    });

    test('do not mutate what they were chained from', () {
      final base = get(label('a'));
      base.post(label('b'));

      expect(base.handlers.keys, ['GET']);
    });

    test('accept an arbitrary method through on', () {
      expect(const MethodRouter().on('purge', label('x')).handlers.keys,
          ['PURGE']);
    });

    test('serve each method from one path', () async {
      final app = Router()
        ..route('/a', get(label('g')).post(label('p')).delete(label('d')));
      final handler = app.handler;

      expect(await (await handler(request('GET', '/a'))).readAsString(), 'g');
      expect(await (await handler(request('POST', '/a'))).readAsString(), 'p');
      expect(
          await (await handler(request('DELETE', '/a'))).readAsString(), 'd');
    });

    test('match a lowercase method from the wire', () async {
      final app = Router()..route('/a', get(label('g')));

      expect((await app.handler(request('get', '/a'))).statusCode, 200);
    });
  });
}
