import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('layer order', () {
    test('runs outermost first, down the tree', () async {
      final log = <String>[];
      final module = todosModule()..layer(tagged('module', log));
      final api = Router()
        ..layer(tagged('api', log))
        ..merge(module);
      final app = Router()
        ..layer(tagged('root', log))
        ..nest('/api', api);

      await app.handler(request('GET', '/api/todos'));

      expect(log, ['root', 'api', 'module']);
    });

    test('runs several layers on one router in declaration order', () async {
      final log = <String>[];
      final app = Router()
        ..layer(tagged('first', log))
        ..layer(tagged('second', log))
        ..route('/a', get(label('a')));

      await app.handler(request('GET', '/a'));

      expect(log, ['first', 'second']);
    });

    test('runs the root layer even when nothing matched', () async {
      final log = <String>[];
      final app = Router()
        ..layer(tagged('root', log))
        ..route('/a', get(label('a')));

      await app.handler(request('GET', '/zzz'));

      expect(log, ['root']);
    });

    test('does not run a nested layer for a sibling route', () async {
      final log = <String>[];
      final api = Router()
        ..layer(tagged('api', log))
        ..route('/inside', get(label('inside')));
      final app = Router()
        ..nest('/api', api)
        ..route('/outside', get(label('outside')));

      await app.handler(request('GET', '/outside'));

      expect(log, isEmpty);
    });

    test('runs a nested layer only once for its own route', () async {
      final log = <String>[];
      final api = Router()
        ..layer(tagged('api', log))
        ..route('/inside', get(label('inside')));
      final app = Router()..nest('/api', api);

      await app.handler(request('GET', '/api/inside'));

      expect(log, ['api']);
    });
  });
}
