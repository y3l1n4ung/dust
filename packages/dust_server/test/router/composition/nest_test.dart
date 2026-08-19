import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('nest', () {
    test('mounts a module under a prefix', () async {
      final app = Router()..nest('/api', todosModule());

      expect(
        await (await app.handler(request('GET', '/api/todos'))).readAsString(),
        'list',
      );
    });

    test('nests to any depth', () async {
      final v1 = Router()..nest('/todos', todosModule(prefix: ''));
      final api = Router()..nest('/v1', v1);
      final app = Router()..nest('/api', api);

      expect(
        (await app.handler(request('GET', '/api/v1/todos'))).statusCode,
        200,
      );
    });

    test('normalizes a prefix written without a slash', () async {
      final app = Router()..nest('api', todosModule());

      expect((await app.handler(request('GET', '/api/todos'))).statusCode, 200);
    });

    test('normalizes a prefix written with a trailing slash', () async {
      final app = Router()..nest('/api/', todosModule());

      expect((await app.handler(request('GET', '/api/todos'))).statusCode, 200);
    });

    test('mounts the same module at two prefixes', () async {
      final module = todosModule();
      final app = Router()
        ..nest('/api/v1', module)
        ..nest('/api/v2', module);
      final handler = app.handler;

      expect((await handler(request('GET', '/api/v1/todos'))).statusCode, 200);
      expect((await handler(request('GET', '/api/v2/todos'))).statusCode, 200);
    });

    test('refuses to nest itself', () {
      final app = Router();

      expect(() => app.nest('/loop', app), throwsArgumentError);
    });
  });

  group('merge', () {
    test('mounts without adding a prefix', () async {
      final app = Router()..merge(todosModule());

      expect((await app.handler(request('GET', '/todos'))).statusCode, 200);
    });

    test('combines two modules at one level', () async {
      final app = Router()
        ..merge(todosModule())
        ..merge(todosModule(prefix: '/notes'));
      final handler = app.handler;

      expect((await handler(request('GET', '/todos'))).statusCode, 200);
      expect((await handler(request('GET', '/notes'))).statusCode, 200);
    });
  });
}
