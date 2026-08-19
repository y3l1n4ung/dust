import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// The `Host` header is sent by the client, so it is data rather than fact.
/// These check the reading; the warning against interpolating it into a URL
/// lives on the extractor, because no test can enforce that.

void main() {
  Request withHost(String? host) => request(
        'GET',
        '/',
        headers: {if (host != null) 'host': host},
      );

  group('the extractor', () {
    test('reads the header', () async {
      final outcome = await const HostExtractable().extract(
        withHost('api.example.test'),
      );

      expect(expectOk(outcome), 'api.example.test');
    });

    test('keeps the port, which is part of the authority', () async {
      final outcome =
          await const HostExtractable().extract(withHost('localhost:8080'));

      expect(expectOk(outcome), 'localhost:8080');
    });

    test('reads it whatever case the header name arrived in', () async {
      final outcome = await const HostExtractable().extract(
        request('GET', '/', headers: const {'Host': 'a.test'}),
      );

      expect(expectOk(outcome), 'a.test');
    });

    test('rejects with 400 when it is absent, since HTTP/1.1 requires it',
        () async {
      expectStatus(await const HostExtractable().extract(withHost(null)), 400);
    });

    test('rejects an empty value the same way', () async {
      expectStatus(await const HostExtractable().extract(withHost('')), 400);
    });

    test('hands back whatever was sent, including a name nobody serves',
        () async {
      // Reading is not vouching. Comparing against names you serve is the
      // caller's job, and the reason the doc says so.
      final outcome =
          await const HostExtractable().extract(withHost('evil.test'));

      expect(expectOk(outcome), 'evil.test');
    });

    test('ignores X-Forwarded-Host, which anything can forge', () async {
      final outcome = await const HostExtractable().extract(
        request(
          'GET',
          '/',
          headers: const {
            'host': 'real.test',
            'x-forwarded-host': 'spoofed.test',
          },
        ),
      );

      expect(expectOk(outcome), 'real.test');
    });
  });

  group('from a request', () {
    test('reads the host', () async {
      expect(await withHost('a.test').host(), 'a.test');
    });

    test('throws the 400 when it is missing', () async {
      expect(
        withHost(null).host,
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 400)),
      );
    });

    test('is reachable through the shortcut too', () async {
      expect(expectOk(await host().extract(withHost('a.test'))), 'a.test');
    });
  });

  group('routing on it', () {
    test('lets a handler serve one tenant and refuse another', () async {
      final app = Router()
        ..route('/', get((request) async {
          final name = await request.host();
          return const {'a.test', 'b.test'}.contains(name)
              ? Ok<Map<String, Object?>, Rejection>({'tenant': name})
              : const Err(Rejection.notFound('no such tenant'));
        }));

      expect((await app.handler(withHost('a.test'))).statusCode, 200);
      expect((await app.handler(withHost('evil.test'))).statusCode, 404);
    });
  });
}
