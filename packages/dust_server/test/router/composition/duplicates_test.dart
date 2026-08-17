import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('duplicate routes', () {
    test('are caught when the handler is built', () {
      final app = Router()
        ..merge(todosModule())
        ..merge(todosModule());

      expect(
        () => app.handler,
        throwsA(
          isA<StateError>().having(
            (error) => error.message,
            'message',
            'duplicate route: GET /todos',
          ),
        ),
      );
    });

    test('are caught within one router', () {
      final app = Router()
        ..route('/a', get(label('first')))
        ..route('/a', get(label('second')));

      expect(() => app.handler, throwsStateError);
    });

    test('are allowed under different prefixes', () {
      final app = Router()
        ..nest('/v1', todosModule())
        ..nest('/v2', todosModule());

      expect(() => app.handler, returnsNormally);
    });

    test('are allowed for different methods on one path', () {
      final app = Router()
        ..route('/a', get(label('g')))
        ..route('/a', post(label('p')));

      expect(() => app.handler, returnsNormally);
    });

    test('distinguish a trailing slash', () {
      final app = Router()
        ..route('/a', get(label('bare')))
        ..route('/a/', get(label('slashed')));

      expect(() => app.handler, returnsNormally);
    });

    test('are caught by describe as well', () {
      final app = Router()
        ..merge(todosModule())
        ..merge(todosModule());

      expect(app.describe, throwsStateError);
    });
  });
}
