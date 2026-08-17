import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('body limit', () {
    test('reaches extractors through the request context', () async {
      int? seen;
      final app = Router(bodyLimit: 64)
        ..route('/a', post((request) async {
          seen = request.context[bodyLimitContextKey] as int?;
          return noContent();
        }));

      await app.handler(request('POST', '/a', body: 'x'));

      expect(seen, 64);
    });

    test('defaults to one mebibyte', () async {
      int? seen;
      final app = Router()
        ..route('/a', post((request) async {
          seen = request.context[bodyLimitContextKey] as int?;
          return noContent();
        }));

      await app.handler(request('POST', '/a', body: 'x'));

      expect(seen, defaultBodyLimit);
    });

    test('refuses an oversized body end to end', () async {
      final app = Router(bodyLimit: 4)
        ..route('/a', post((request) async {
          final outcome = await const RawBodyExtractable().extract(request);
          return switch (outcome) {
            Ok() => noContent(),
            Err(:final error) => error.intoResponse(),
          };
        }));

      final response =
          await app.handler(request('POST', '/a', body: 'far too long'));

      expect(response.statusCode, 413);
    });

    test('overrides a limit the extractor was built with', () async {
      final app = Router(bodyLimit: 2)
        ..route('/a', post((request) async {
          final outcome =
              await const RawBodyExtractable(limit: 1024).extract(request);
          return switch (outcome) {
            Ok() => noContent(),
            Err(:final error) => error.intoResponse(),
          };
        }));

      expect(
        (await app.handler(request('POST', '/a', body: 'too long'))).statusCode,
        413,
      );
    });
  });
}
