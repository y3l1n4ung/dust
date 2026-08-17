import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('parameter constraints', () {
    test('match only what the pattern allows', () async {
      final app = Router()..route('/todos/{id|[0-9]+}', get(label('numeric')));
      final handler = app.handler;

      expect((await handler(request('GET', '/todos/7'))).statusCode, 200);
      expect((await handler(request('GET', '/todos/seven'))).statusCode, 404);
    });

    test('still capture the value', () async {
      late Map<String, String> seen;
      final app = Router()
        ..route('/todos/{id|[0-9]+}', get((request) async {
          seen = RequestParts.of(request).pathParameters;
          return noContent();
        }));

      await app.handler(request('GET', '/todos/42'));

      expect(seen, {'id': '42'});
    });

    test('let a looser route pick up what they reject', () async {
      final app = Router()
        ..route('/todos/{id|[0-9]+}', get(label('numeric')))
        ..route('/todos/{slug}', get(label('slug')));
      final handler = app.handler;

      expect(
        await (await handler(request('GET', '/todos/7'))).readAsString(),
        'numeric',
      );
      expect(
        await (await handler(request('GET', '/todos/hello'))).readAsString(),
        'slug',
      );
    });

    test('span a slash when the pattern allows it', () async {
      // The pattern is inserted as written, so `.+` crosses segments. That is
      // the caller's choice; `[^/]+` is the way to stay in one segment.
      final app = Router()..route('/todos/{id|.+}', get(label('any')));

      expect((await app.handler(request('GET', '/todos/a/b'))).statusCode, 200);
    });

    test('stay in one segment with an anchored pattern', () async {
      final app = Router()..route(r'/todos/{id|[^/]+}', get(label('one')));

      expect((await app.handler(request('GET', '/todos/a/b'))).statusCode, 404);
    });
  });
}
