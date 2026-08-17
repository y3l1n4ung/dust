import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('conflicting routes', () {
    test('reject an exact duplicate', () {
      final app = Router()
        ..route('/a', get(label('first')))
        ..route('/a', get(label('second')));

      expect(
        () => app.handler,
        throwsA(
          isA<StateError>().having(
            (error) => error.message,
            'message',
            'duplicate route: GET /a',
          ),
        ),
      );
    });

    test('reject two parameters that differ only in name', () {
      final app = Router()
        ..route('/a/{id}', get(label('id')))
        ..route('/a/{name}', get(label('name')));

      expect(
        () => app.handler,
        throwsA(
          isA<StateError>().having(
            (error) => error.message,
            'message',
            'unreachable route: GET /a/{name} is shadowed by GET /a/{id}',
          ),
        ),
      );
    });

    test('reject a shadowed route across modules', () {
      final app = Router()
        ..merge(Router.module(
          prefix: '/todos',
          routes: [Route('GET', '/{id}', label('module'))],
        ))
        ..route('/todos/{key}', get(label('handwritten')));

      expect(() => app.handler, throwsStateError);
    });

    test('allow the same pattern under different methods', () {
      final app = Router()
        ..route('/a/{id}', get(label('read')))
        ..route('/a/{key}', post(label('write')));

      expect(() => app.handler, returnsNormally);
    });

    test('allow parameters in different positions', () {
      final app = Router()
        ..route('/a/{id}/b', get(label('one')))
        ..route('/a/b/{id}', get(label('two')));

      expect(() => app.handler, returnsNormally);
    });

    test('allow a constrained parameter beside a loose one', () {
      final app = Router()
        ..route('/a/{id|[0-9]+}', get(label('numeric')))
        ..route('/a/{slug}', get(label('slug')));

      expect(() => app.handler, returnsNormally);
    });

    test('reject two mounts on one prefix', () {
      final app = Router()
        ..mount('/m', label('first'))
        ..mount('/m', label('second'));

      expect(() => app.handler, throwsStateError);
    });

    test('are reported by describe as well', () {
      final app = Router()
        ..route('/a/{id}', get(label('id')))
        ..route('/a/{name}', get(label('name')));

      expect(app.describe, throwsStateError);
    });
  });

  group('route registration', () {
    test('refuses a MethodRouter with no handlers', () {
      final app = Router();

      expect(
        () => app.route('/a', const MethodRouter()),
        throwsA(
          isA<ArgumentError>().having(
            (error) => error.message,
            'message',
            'route "/a" was given no handlers',
          ),
        ),
      );
    });

    test('accepts a MethodRouter with one handler', () {
      final app = Router();

      expect(() => app.route('/a', get(label('a'))), returnsNormally);
    });
  });
}
