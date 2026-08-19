import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('sealing', () {
    test('caches the handler', () {
      final app = Router()..merge(todosModule());

      expect(identical(app.handler, app.handler), isTrue);
    });

    test('refuses a route added afterwards', () {
      final app = Router()..route('/a', get(label('a')));
      app.handler;

      expect(() => app.route('/b', get(label('b'))), throwsStateError);
    });

    test('refuses a nest afterwards', () {
      final app = Router()..route('/a', get(label('a')));
      app.handler;

      expect(() => app.nest('/late', todosModule()), throwsStateError);
    });

    test('refuses a layer afterwards', () {
      final app = Router()..route('/a', get(label('a')));
      app.handler;

      expect(() => app.layer(const HeaderLayer('x', 'y')), throwsStateError);
    });

    test('refuses state afterwards', () {
      final app = Router()..route('/a', get(label('a')));
      app.handler;

      expect(() => app.withState('late'), throwsStateError);
    });

    test('refuses a fallback afterwards', () {
      final app = Router()..route('/a', get(label('a')));
      app.handler;

      expect(() => app.fallback(label('late')), throwsStateError);
    });

    test('seals nested routers too', () {
      final inner = Router()..route('/a', get(label('a')));
      final app = Router()..nest('/api', inner);
      app.handler;

      expect(() => inner.route('/b', get(label('b'))), throwsStateError);
    });

    test('allows building the tree before the handler is read', () {
      final inner = Router();
      final app = Router()..nest('/api', inner);
      inner.route('/a', get(label('a')));

      expect(() => app.handler, returnsNormally);
    });
  });
}
