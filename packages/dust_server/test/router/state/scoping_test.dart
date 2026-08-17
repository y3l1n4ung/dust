import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

final class _Repo {
  const _Repo(this.name);

  final String name;
}

final class _Other {
  const _Other(this.name);

  final String name;
}

Handler _reads<T extends Object>(void Function(Result<T, Rejection>) capture) {
  return (request) async {
    capture(await StateExtractable<T>().extract(request));
    return noContent();
  };
}

void main() {
  group('state scoping', () {
    test('is readable from a route on the same router', () async {
      late Result<_Repo, Rejection> seen;
      final app = Router()
        ..route('/a', get(_reads<_Repo>((value) => seen = value)))
        ..withState(const _Repo('same'));

      await app.handler(request('GET', '/a'));

      expect(expectOk(seen).name, 'same');
    });

    test('flows down into nested routers', () async {
      late Result<_Repo, Rejection> seen;
      final inner = Router()
        ..route('/a', get(_reads<_Repo>((value) => seen = value)));
      final app = Router()
        ..nest('/api', inner)
        ..withState(const _Repo('outer'));

      await app.handler(request('GET', '/api/a'));

      expect(expectOk(seen).name, 'outer');
    });

    test('is overridden by an inner router', () async {
      late Result<_Repo, Rejection> seen;
      final inner = Router()
        ..route('/a', get(_reads<_Repo>((value) => seen = value)))
        ..withState(const _Repo('inner'));
      final app = Router()
        ..nest('/api', inner)
        ..withState(const _Repo('outer'));

      await app.handler(request('GET', '/api/a'));

      expect(expectOk(seen).name, 'inner');
    });

    test('keeps different types apart', () async {
      late Result<_Other, Rejection> seen;
      final app = Router()
        ..route('/a', get(_reads<_Other>((value) => seen = value)))
        ..withState(const _Repo('repo'))
        ..withState(const _Other('other'));

      await app.handler(request('GET', '/a'));

      expect(expectOk(seen).name, 'other');
    });

    test('does not leak into a sibling branch', () async {
      late Result<_Repo, Rejection> seen;
      final withIt = Router()
        ..route('/yes', get(label('yes')))
        ..withState(const _Repo('scoped'));
      final without = Router()
        ..route('/no', get(_reads<_Repo>((value) => seen = value)));
      final app = Router()
        ..nest('/with', withIt)
        ..nest('/without', without);

      await app.handler(request('GET', '/without/no'));

      expectStatus(seen, 500);
    });
  });
}
