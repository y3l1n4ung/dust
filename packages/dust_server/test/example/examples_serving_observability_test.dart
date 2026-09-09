import 'dart:io';
import 'package:dust_server/server.dart';
import 'package:test/test.dart';
import '../../example/metrics.dart' as metrics;
import '../../example/sessions.dart' as sessions;
import '../../example/testing.dart' as testing;
import 'serve.dart';
import 'support.dart';

/// Running the server: isolates, shutdown, TLS and observability.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
/// Metrics, sessions, health checks and request logging.
void main() {
  group('metrics', () {
    test('two URLs on one route share one series', () async {
      // The reason to label by matched route. Labelling by path gives one time
      // series per order, which is how a metrics backend falls over.
      final collected = metrics.Metrics();
      final app = await example(metrics.buildApp(metrics: collected));

      await app.get('/orders/41');
      await app.get('/orders/42');

      final scraped = (await app.get('/metrics')).body;

      expect(
        scraped,
        contains('http_requests_total{route="/orders/{id}",'
            'method="GET",status="200"} 2'),
      );
      expect(scraped, isNot(contains('/orders/41')));
    });

    test('a 404 collapses to one series, not one per probed path', () async {
      // An unmatched path is client-controlled. A scanner walking your URLs
      // would otherwise create a series each.
      final collected = metrics.Metrics();
      final app = await example(metrics.buildApp(metrics: collected));

      await app.get('/nothing');
      await app.get('/also-nothing');

      expect(
        (await app.get('/metrics')).body,
        contains('http_requests_total{route="<unmatched>",'
            'method="GET",status="404"} 2'),
      );
    });

    test('the histogram buckets are cumulative', () async {
      final collected = metrics.Metrics();
      final app = await example(metrics.buildApp(metrics: collected));

      await app.get('/orders/41');
      final scraped = (await app.get('/metrics')).body;

      final infinite =
          RegExp(r'le="\+Inf"\} (\d+)').firstMatch(scraped)!.group(1);

      expect(infinite, '1');
    });

    test('the scrape endpoint is plain text, as Prometheus expects', () async {
      final app = await example(metrics.buildApp());

      final response = await app.get('/metrics');

      expect(response.headers['content-type'], startsWith('text/plain'));
      expect(response.body, contains('# TYPE http_requests_total counter'));
    });
  });

  group('sessions', () {
    const secret = 'a-test-secret-that-is-long-enough-to-use';

    test('signing in sets a signed cookie with every attribute', () async {
      final app = await example(sessions.buildApp(secret: secret));

      final response = await app.send(
        'POST',
        '/sign-in',
        body: 'user=ada',
        headers: const {'content-type': 'application/x-www-form-urlencoded'},
      );
      final cookie = response.headers['set-cookie']!;

      expect(cookie, contains('HttpOnly'));
      expect(cookie, contains('Secure'));
      expect(cookie, contains('SameSite=Lax'));
    });

    test('the cookie round-trips to the user it names', () async {
      final app = await example(sessions.buildApp(secret: secret));
      final signer = sessions.Sessions(secret);

      final response = await app.get(
        '/me',
        headers: {'cookie': 'session=${valueOf(signer.cookieFor("ada"))}'},
      );

      expect(app.object(response), {'user': 'ada'});
    });

    test('no cookie is a 401', () async {
      final app = await example(sessions.buildApp(secret: secret));

      expect((await app.get('/me')).statusCode, 401);
    });

    test('a tampered payload is refused', () async {
      // Signed, so the payload cannot be edited — a user cannot promote
      // themselves by rewriting the cookie.
      final app = await example(sessions.buildApp(secret: secret));
      final signer = sessions.Sessions(secret);
      final valid = valueOf(signer.cookieFor('ada'));
      final forged = 'ZZZ${valid.substring(3)}';

      expect(
        (await app.get('/me', headers: {'cookie': 'session=$forged'}))
            .statusCode,
        401,
      );
    });

    test('a cookie signed with another secret is refused', () async {
      final app = await example(sessions.buildApp(secret: secret));
      final other = sessions.Sessions('a-completely-different-secret-value!!');

      expect(
        (await app.get(
          '/me',
          headers: {'cookie': 'session=${valueOf(other.cookieFor("ada"))}'},
        ))
            .statusCode,
        401,
      );
    });

    test('an expired session is refused even though the signature is good',
        () async {
      // Max-Age is a hint to the browser. A client can keep sending an expired
      // cookie forever, so the expiry has to be signed and checked here.
      final app = await example(sessions.buildApp(secret: secret));
      final expired = sessions.Sessions(
        secret,
        lifetime: const Duration(days: -1),
      );

      expect(
        (await app.get(
          '/me',
          headers: {'cookie': 'session=${valueOf(expired.cookieFor("ada"))}'},
        ))
            .statusCode,
        401,
      );
    });

    test('a malformed cookie is refused rather than crashing', () async {
      final app = await example(sessions.buildApp(secret: secret));

      for (final value in ['', 'nodot', 'a.b.c', 'not-base64.signature']) {
        expect(
          (await app.get('/me', headers: {'cookie': 'session=$value'}))
              .statusCode,
          401,
          reason: 'cookie "$value"',
        );
      }
    });

    test('signing out expires the cookie', () async {
      final app = await example(sessions.buildApp(secret: secret));

      final response = await app.send('POST', '/sign-out');

      expect(response.headers['set-cookie'], contains('Max-Age=0'));
    });
  });

  group('testing', () {
    test('the in-process handler needs no socket', () async {
      // Fast enough to run thousands of, and right for statuses and bodies.
      final app = testing.buildApp(testing.NoteStore(['only']));

      final response = await app.handler(
        Request('GET', Uri.parse('http://localhost/notes')),
      );

      expect(response.statusCode, 200);
      expect(await response.readAsString(), '["only"]');
    });

    test('the injected store is what the test asserts on', () async {
      final store = testing.NoteStore([]);
      final app = await example(testing.buildApp(store));

      await app.post('/notes', const {'title': 'written'});

      expect(store.titles, ['written']);
    });

    test('a socket catches what the wire does', () async {
      // gzip only exists on a socket. The in-process handler would return the
      // uncompressed body and the assertion would prove nothing.
      final store = testing.NoteStore(
        List.generate(80, (index) => 'a note with a reasonably long title'),
      );
      final app = await example(testing.buildApp(store));

      final client = HttpClient()..autoUncompress = false;
      addTearDown(client.close);
      final request = await client.getUrl(app.uri('/notes'));
      request.headers.set('accept-encoding', 'gzip');
      final response = await request.close();
      await response.drain<void>();

      expect(response.headers.value('content-encoding'), 'gzip');
    });

    test('a missing note is a 404 from the Result, not a throw', () async {
      final app = await example(testing.buildApp(testing.NoteStore([])));

      expect((await app.get('/notes/1')).statusCode, 404);
    });
  });
}
