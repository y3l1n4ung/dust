import 'package:test/test.dart';
import '../../example/bearer_auth.dart' as bearer_auth;
import '../../example/cookies.dart' as cookies;
import '../../example/credential_schemes.dart' as credential_schemes;
import 'serve.dart';

/// Extractors, and what happens when one cannot produce a value.
///
/// Split out of one suite per example so each file stays readable; they
/// share the harness in `support.dart`.
/// Cookies, bearer tokens and credential schemes.
void main() {
  group('cookies', () {
    test('sign-in sets a cookie with every attribute that matters', () async {
      final app = await example(cookies.buildApp());

      final header = (await app.get('/sign-in')).headers['set-cookie']!;

      expect(header, contains('user=ada'));
      expect(header, contains('HttpOnly'));
      expect(header, contains('Secure'));
      expect(header, contains('SameSite=Lax'));
      expect(header, contains('Path=/'));
    });

    test('sign-out expires the same cookie, since HTTP has no delete',
        () async {
      final app = await example(cookies.buildApp());

      final header = (await app.get('/sign-out')).headers['set-cookie']!;

      expect(header, contains('Max-Age=0'));
    });

    test('one cookie is read, and absent is null rather than an error',
        () async {
      final app = await example(cookies.buildApp());

      expect(
        app.object(await app.get('/whoami', headers: {'cookie': 'user=ada'})),
        {'user': 'ada'},
      );
      expect(app.object(await app.get('/whoami')), {'user': null});
    });

    test('the whole jar is read at once', () async {
      final app = await example(cookies.buildApp());

      final response = await app.get('/all', headers: {'cookie': 'a=1; b=2'});

      expect(app.object(response)['cookies'], {'a': '1', 'b': '2'});
    });
  });

  group('bearer_auth', () {
    test('a known token names its user', () async {
      final app = await example(bearer_auth.buildApp());

      final response = await app.get(
        '/me',
        headers: {'authorization': 'Bearer t-ada'},
      );

      expect(app.object(response), {'user': 'ada'});
    });

    test('no credential is 401 with the challenge', () async {
      final app = await example(bearer_auth.buildApp());

      final response = await app.get('/me');

      expect(response.statusCode, 401);
      expect(response.headers['www-authenticate'], contains('Bearer'));
    });

    test('the wrong scheme is 401, not 403', () async {
      final app = await example(bearer_auth.buildApp());

      final response = await app.get(
        '/me',
        headers: {'authorization': 'Basic YWRhOnNlY3JldA=='},
      );

      expect(response.statusCode, 401);
    });

    test('a real credential that is not allowed is 403', () async {
      // A client that retries on 401 loops forever if a wrong token answers
      // 401. The distinction is what tells it to stop.
      final app = await example(bearer_auth.buildApp());

      final response = await app.get(
        '/me',
        headers: {'authorization': 'Bearer nope'},
      );

      expect(response.statusCode, 403);
    });

    test('a route that asks for no credential requires none', () async {
      final app = await example(bearer_auth.buildApp());

      expect((await app.get('/public')).statusCode, 200);
    });
  });

  group('credential_schemes', () {
    test('a bearer token is accepted', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        app.object(
          await app.get('/whoami', headers: {'authorization': 'Bearer t-ada'}),
        ),
        {'via': 'bearer', 'id': 'ada'},
      );
    });

    test('an API key header is accepted', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        app.object(await app.get('/whoami', headers: {'x-api-key': 'k-robot'})),
        {'via': 'api-key', 'id': 'robot'},
      );
    });

    test('HTTP Basic is accepted', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        app.object(
          await app.get(
            '/whoami',
            headers: {'authorization': 'Basic YWRhOnNlY3JldA=='},
          ),
        ),
        {'via': 'basic', 'id': 'ada'},
      );
    });

    test('a session cookie is accepted', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        app.object(
            await app.get('/whoami', headers: {'cookie': 'session=s-ada'})),
        {'via': 'session', 'id': 'ada'},
      );
    });

    test('a key in the query string is refused, because URLs leak', () async {
      // Query strings reach access logs, proxy logs, browser history, and the
      // Referer header. allowQuery: false is the setting that closes it.
      final app = await example(credential_schemes.buildApp());

      expect((await app.get('/whoami?api_key=k-robot')).statusCode, 401);
    });

    test('no credential at all is 401 offering every scheme', () async {
      // It used to name only whichever scheme ran last — accurate for that one
      // and misleading about the other three.
      final app = await example(credential_schemes.buildApp());

      final response = await app.get('/whoami');

      expect(response.statusCode, 401);
      final challenge = response.headers['www-authenticate']!;
      expect(challenge, contains('Bearer'));
      expect(challenge, contains('Cookie'));
      expect(app.object(response)['error'], 'no credentials were supplied');
    });

    test('a wrong Basic password is refused', () async {
      final app = await example(credential_schemes.buildApp());

      expect(
        (await app.get(
          '/whoami',
          headers: {'authorization': 'Basic YWRhOnd)cm9uZw=='},
        ))
            .statusCode,
        401,
      );
    });
  });
}
