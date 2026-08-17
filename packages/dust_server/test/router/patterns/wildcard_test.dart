import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

Handler _capture(void Function(Map<String, String>) capture) {
  return (request) async {
    capture(RequestParts.of(request).pathParameters);
    return noContent();
  };
}

void main() {
  group('catch-all segments', () {
    test('swallow the rest of the path', () async {
      late Map<String, String> seen;
      final app = Router()
        ..route('/files/{*rest}', get(_capture((p) => seen = p)));

      await app.handler(request('GET', '/files/a/b/c.txt'));

      expect(seen, {'rest': 'a/b/c.txt'});
    });

    test('match a single segment too', () async {
      late Map<String, String> seen;
      final app = Router()
        ..route('/files/{*rest}', get(_capture((p) => seen = p)));

      await app.handler(request('GET', '/files/a.txt'));

      expect(seen, {'rest': 'a.txt'});
    });

    test('match an empty remainder', () async {
      late Map<String, String> seen;
      final app = Router()
        ..route('/files/{*rest}', get(_capture((p) => seen = p)));

      final response = await app.handler(request('GET', '/files/'));

      expect(response.statusCode, 204);
      expect(seen, {'rest': ''});
    });

    test('combine with earlier parameters', () async {
      late Map<String, String> seen;
      final app = Router()
        ..route('/u/{user}/files/{*rest}', get(_capture((p) => seen = p)));

      await app.handler(request('GET', '/u/ada/files/x/y'));

      expect(seen, {'user': 'ada', 'rest': 'x/y'});
    });

    test('lose to an exact route declared first', () async {
      final app = Router()
        ..route('/files/readme', get(label('exact')))
        ..route('/files/{*rest}', get(label('catch-all')));

      expect(
        await (await app.handler(request('GET', '/files/readme')))
            .readAsString(),
        'exact',
      );
    });

    test('are rejected anywhere but last', () {
      expect(
        () => compilePath('/files/{*rest}/more'),
        throwsArgumentError,
      );
    });
  });
}
