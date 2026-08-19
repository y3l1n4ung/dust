import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('trailing slashes', () {
    test('a trailing slash is a different path', () async {
      final app = Router()..route('/a', get(label('a')));
      final handler = app.handler;

      expect((await handler(request('GET', '/a'))).statusCode, 200);
      expect((await handler(request('GET', '/a/'))).statusCode, 404);
    });

    test('both can be served when both are declared', () async {
      final app = Router()
        ..route('/a', get(label('bare')))
        ..route('/a/', get(label('slashed')));
      final handler = app.handler;

      expect(
          await (await handler(request('GET', '/a'))).readAsString(), 'bare');
      expect(
        await (await handler(request('GET', '/a/'))).readAsString(),
        'slashed',
      );
    });

    test('a repeated slash does not collapse', () async {
      final app = sampleRouter().handler;

      expect((await app(request('GET', '/a//7'))).statusCode, 404);
    });
  });
}
