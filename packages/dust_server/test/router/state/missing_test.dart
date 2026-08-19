import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

final class _Repo {
  const _Repo();
}

void main() {
  group('missing state', () {
    test('is a 500, not a client error', () async {
      final outcome =
          await const StateExtractable<_Repo>().extract(request('GET', '/'));

      expectStatus(outcome, 500);
    });

    test('names the type and the fix', () async {
      final outcome =
          await const StateExtractable<_Repo>().extract(request('GET', '/'));

      final rejection = expectStatus(outcome, 500);
      expect(rejection.message, contains('_Repo'));
      expect(rejection.message, contains('withState'));
    });

    test('surfaces as a 500 response end to end', () async {
      final app = Router()
        ..route('/a', get((request) async {
          final outcome =
              await const StateExtractable<_Repo>().extract(request);
          return switch (outcome) {
            Ok() => noContent(),
            Err(:final error) => error.intoResponse(),
          };
        }));

      expect((await app.handler(request('GET', '/a'))).statusCode, 500);
    });

    test('is not collapsed by Option', () async {
      final outcome = await const OptionalExtractable(
        StateExtractable<_Repo>(),
      ).extract(request('GET', '/'));

      expectStatus(outcome, 500);
    });
  });
}
