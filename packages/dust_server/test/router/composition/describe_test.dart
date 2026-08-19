import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

final class _Doc {
  const _Doc(this.summary);

  final String summary;
}

final class _Owner {
  const _Owner(this.team);

  final String team;
}

void main() {
  group('describe', () {
    test('lists every route with its served path', () {
      final app = Router()..nest('/api/v1', todosModule());

      expect(
        app.describe().map((route) => '${route.method} ${route.path}'),
        containsAll(<String>[
          'GET /api/v1/todos',
          'GET /api/v1/todos/{id}',
          'POST /api/v1/todos',
        ]),
      );
    });

    test('keeps placeholders unrewritten', () {
      final app = Router()..merge(todosModule());

      expect(
        app.describe().map((route) => route.path),
        contains('/todos/{id}'),
      );
    });

    test('carries metadata outermost first', () {
      final api = Router(metadata: const _Owner('core'))
        ..merge(todosModule(metadata: const _Doc('todos')));
      final app = Router()..nest('/api', api);

      final route = app.describe().first;

      expect(route.metadata.first, isA<_Owner>());
      expect(route.metadata.last, isA<_Doc>());
    });

    test('finds metadata by type, innermost first', () {
      final api = Router(metadata: const _Owner('core'))
        ..merge(todosModule(metadata: const _Owner('todos')));
      final app = Router()..nest('/api', api);

      expect(app.describe().first.metadataOf<_Owner>()?.team, 'todos');
    });

    test('returns null for a type nobody attached', () {
      final app = Router()..merge(todosModule());

      expect(app.describe().first.metadataOf<_Doc>(), isNull);
    });

    test('omits routers that attached nothing', () {
      final app = Router()..merge(todosModule());

      expect(app.describe().first.metadata, isEmpty);
    });

    test('covers hand-written routes as well as modules', () {
      final app = Router()
        ..route('/health', get(label('health')))
        ..merge(todosModule());

      expect(
        app.describe().map((route) => route.path),
        containsAll(<String>['/health', '/todos']),
      );
    });

    test('describes itself readably', () {
      final app = Router()..route('/a', get(label('a')));

      expect(app.describe().single.toString(), 'MountedRoute(GET /a)');
    });
  });
}
