import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('placeholders', () {
    test('capture one segment', () async {
      final app = sampleRouter().handler;

      expect(
          await (await app(request('GET', '/a/7'))).readAsString(), '/a/{id}');
    });

    test('do not span a slash', () async {
      final app = sampleRouter().handler;

      expect((await app(request('GET', '/a/7/8'))).statusCode, 404);
    });

    test('capture several in one path', () async {
      late Map<String, String> seen;
      final app = Router()
        ..route('/a/{id}/b/{other}', get((request) async {
          seen = RequestParts.of(request).pathParameters;
          return noContent();
        }));

      await app.handler(request('GET', '/a/7/b/9'));

      expect(seen, {'id': '7', 'other': '9'});
    });

    test('leave the context empty for a static route', () async {
      late Map<String, String> seen;
      final app = Router()
        ..route('/health', get((request) async {
          seen = RequestParts.of(request).pathParameters;
          return noContent();
        }));

      await app.handler(request('GET', '/health'));

      expect(seen, isEmpty);
    });

    test('lose to a static route declared first', () async {
      final app = Router()
        ..route('/a/fixed', get(label('static')))
        ..route('/a/{id}', get(label('dynamic')));

      expect(
        await (await app.handler(request('GET', '/a/fixed'))).readAsString(),
        'static',
      );
    });

    test('win when no static route matches', () async {
      final app = Router()
        ..route('/a/fixed', get(label('static')))
        ..route('/a/{id}', get(label('dynamic')));

      expect(
        await (await app.handler(request('GET', '/a/other'))).readAsString(),
        'dynamic',
      );
    });

    test('reject a missing parameter at extraction, not at matching', () async {
      final outcome = await const PathExtractable<String>('id')
          .extract(request('GET', '/a/7'));

      expectStatus(outcome, 400);
    });
  });
}
