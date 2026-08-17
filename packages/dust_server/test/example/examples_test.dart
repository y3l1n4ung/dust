import 'package:test/test.dart';

import '../../example/hello_world.dart' as hello_world;
import 'serve.dart';

/// One suite for every one-question example.
///
/// Each example gets the shortest test that proves the thing it demonstrates,
/// and they share one harness. Thirty examples with thirty harnesses is thirty
/// places for a leaked socket to hide; more to the point, an example nothing
/// serves is an example that quietly stops compiling.

void main() {
  group('hello_world', () {
    test('answers the root as text, not as a quoted JSON string', () async {
      final app = await example(hello_world.buildApp());

      final response = await app.get('/');

      expect(response.statusCode, 200);
      expect(response.body, 'Hello, world!');
      expect(response.headers['content-type'], startsWith('text/plain'));
    });

    test('reads the name out of the path', () async {
      final app = await example(hello_world.buildApp());

      expect((await app.get('/hello/ada')).body, 'Hello, ada!');
    });

    test('encodes a returned map as JSON without being asked', () async {
      final app = await example(hello_world.buildApp());

      final response = await app.get('/json');

      expect(response.headers['content-type'], 'application/json');
      expect(app.object(response), {'greeting': 'Hello, world!'});
    });

    test('an unmatched path answers 404 without a fallback', () async {
      final app = await example(hello_world.buildApp());

      expect((await app.get('/nope')).statusCode, 404);
    });
  });
}
