import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

Future<String> _captured(String path) async {
  late String seen;
  final app = Router()
    ..route('/files/{name}', get((request) async {
      final outcome =
          await const PathExtractable<String>('name').extract(request);
      seen = switch (outcome) {
        Ok(:final value) => value,
        Err(:final error) => 'REJECTED:${error.status}',
      };
      return noContent();
    }));

  await app.handler(request('GET', path));
  return seen;
}

void main() {
  group('percent encoding', () {
    test('a space arrives decoded', () async {
      expect(await _captured('/files/a%20b'), 'a b');
    });

    test('an encoded slash stays inside one segment', () async {
      expect(await _captured('/files/a%2Fb'), 'a/b');
    });

    test('UTF-8 arrives decoded', () async {
      expect(await _captured('/files/caf%C3%A9'), 'café');
    });

    test('a plus sign is not treated as a space', () async {
      expect(await _captured('/files/a+b'), 'a+b');
    });

    test('malformed encoding arrives literal, not as a crash', () async {
      // Uri.parse re-escapes an invalid sequence, so `%zz` reaches the handler
      // as text rather than failing to decode.
      expect(await _captured('/files/%zz'), '%zz');
    });

    test('a doubly encoded value decodes exactly once', () async {
      expect(await _captured('/files/a%2520b'), 'a%20b');
    });
  });
}
