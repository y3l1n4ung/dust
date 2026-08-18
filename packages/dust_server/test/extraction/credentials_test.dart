import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// The runtime extracts credentials and stops there. What a caller *is*, and
/// what a scope permits, belongs to the application — so what is tested here
/// is reading the credential off the wire and choosing the right refusal, not
/// any notion of identity.

final class _Always implements FromRequestParts<String> {
  const _Always(this.value);

  final String value;

  @override
  Future<Result<String, Rejection>> extract(Request request) async => Ok(value);
}

final class _Refuses implements FromRequestParts<String> {
  const _Refuses(this.status, this.message);

  final int status;
  final String message;

  @override
  Future<Result<String, Rejection>> extract(Request request) async =>
      Err(Rejection.status(status, message));
}

/// Refuses with a 401 carrying [challenge], the way a real scheme does.
final class _Challenges implements FromRequestParts<String> {
  const _Challenges(this.challenge);

  final String challenge;

  @override
  Future<Result<String, Rejection>> extract(Request request) async =>
      Err(Rejection.unauthorized('needs a credential', challenge: challenge));
}

void main() {
  group('an API key', () {
    test('is read from the header', () async {
      final outcome = await const ApiKeyExtractable()
          .extract(request('GET', '/', headers: const {'x-api-key': 'k'}));

      expect(expectOk(outcome), 'k');
    });

    test('is read from the header whatever its case', () async {
      final outcome = await const ApiKeyExtractable()
          .extract(request('GET', '/', headers: const {'X-API-Key': 'k'}));

      expect(expectOk(outcome), 'k');
    });

    test('falls back to the query string', () async {
      final outcome = await const ApiKeyExtractable()
          .extract(request('GET', '/?api_key=k'));

      expect(expectOk(outcome), 'k');
    });

    test('prefers the header over the query', () async {
      final outcome = await const ApiKeyExtractable().extract(
        request('GET', '/?api_key=query', headers: const {'x-api-key': 'head'}),
      );

      expect(expectOk(outcome), 'head');
    });

    test('ignores an empty header and takes the query', () async {
      final outcome = await const ApiKeyExtractable().extract(
        request('GET', '/?api_key=k', headers: const {'x-api-key': ''}),
      );

      expect(expectOk(outcome), 'k');
    });

    test('can refuse the query fallback entirely', () async {
      // A key in a URL lands in access logs and browser history, so a
      // deployment that cannot afford that turns the fallback off.
      const extractor = ApiKeyExtractable(allowQuery: false);
      final outcome = await extractor.extract(request('GET', '/?api_key=k'));

      expectStatus(outcome, 401);
    });

    test('reads the names it was given', () async {
      const extractor = ApiKeyExtractable(header: 'x-token', query: 'token');
      final outcome = await extractor
          .extract(request('GET', '/', headers: const {'x-token': 'k'}));

      expect(expectOk(outcome), 'k');
    });

    test('rejects with 401 when nothing carries a key', () async {
      final outcome =
          await const ApiKeyExtractable().extract(request('GET', '/'));

      expectStatus(outcome, 401);
    });

    test('names the header it wanted', () async {
      final outcome =
          await const ApiKeyExtractable().extract(request('GET', '/'));

      expect(expectErr(outcome).message, contains('x-api-key'));
    });
  });

  group('a session id', () {
    Request withCookie(String value) =>
        request('GET', '/', headers: {'cookie': value});

    test('is read from the cookie', () async {
      final outcome =
          await const SessionIdExtractable().extract(withCookie('session=s1'));

      expect(expectOk(outcome), 's1');
    });

    test('is picked out of a cookie header carrying others', () async {
      final outcome = await const SessionIdExtractable()
          .extract(withCookie('theme=dark; session=s1; a=1'));

      expect(expectOk(outcome), 's1');
    });

    test('reads the cookie name it was given', () async {
      const extractor = SessionIdExtractable(name: 'sid');
      final outcome = await extractor.extract(withCookie('sid=s1'));

      expect(expectOk(outcome), 's1');
    });

    test('rejects with 401, not the 400 a missing cookie would give', () async {
      // "You are not signed in" is not "your request was malformed".
      final outcome =
          await const SessionIdExtractable().extract(request('GET', '/'));

      expectStatus(outcome, 401);
    });

    test('rejects an empty session cookie', () async {
      final outcome =
          await const SessionIdExtractable().extract(withCookie('session='));

      expectStatus(outcome, 401);
    });
  });

  group('the first that succeeds', () {
    test('takes the first when it succeeds', () async {
      const extractor = FirstOf<String>([_Always('a'), _Always('b')]);

      expect(expectOk(await extractor.extract(request('GET', '/'))), 'a');
    });

    test('moves on past a refusal', () async {
      const extractor = FirstOf<String>([_Refuses(401, 'no'), _Always('b')]);

      expect(expectOk(await extractor.extract(request('GET', '/'))), 'b');
    });

    test('offers every scheme challenge when none had a credential', () async {
      // A 401 naming only the last scheme tried tells a browser to do the wrong
      // thing. HTTP allows several challenges in one response.
      const extractor = FirstOf<String>([
        _Challenges('Bearer'),
        _Challenges('Basic realm="api"'),
        _Challenges('Cookie'),
      ]);

      final rejection = expectErr(await extractor.extract(request('GET', '/')));

      expect(rejection.status, 401);
      expect(rejection.challenge, 'Bearer, Basic realm="api", Cookie');
      expect(rejection.message, 'no credentials were supplied');
    });

    test('does not repeat a challenge two schemes share', () async {
      const extractor = FirstOf<String>([
        _Challenges('Bearer'),
        _Challenges('Bearer'),
      ]);

      expect(
        expectErr(await extractor.extract(request('GET', '/'))).challenge,
        'Bearer',
      );
    });

    test('a refusal that is not a 401 wins, because it is more specific',
        () async {
      // A 403 means a credential was presented and was not good enough. Burying
      // it under "no credentials" sends the caller after the wrong problem.
      const extractor = FirstOf<String>([
        _Refuses(403, 'that key is not allowed here'),
        _Challenges('Cookie'),
      ]);

      final rejection = expectErr(await extractor.extract(request('GET', '/')));

      expect(rejection.status, 403);
      expect(rejection.message, 'that key is not allowed here');
    });

    test('a challenge carrying its own parameters stays parseable', () async {
      // `Basic realm="x", charset="UTF-8"` is one challenge with a comma in it.
      // Joined with others the header is still unambiguous, because an
      // auth-param contains `=` and a scheme name does not — so a parser
      // attaches charset to Basic rather than reading it as a scheme.
      const extractor = FirstOf<String>([
        _Challenges('Basic realm="api", charset="UTF-8"'),
        _Challenges('Cookie'),
      ]);

      expect(
        expectErr(await extractor.extract(request('GET', '/'))).challenge,
        'Basic realm="api", charset="UTF-8", Cookie',
      );
    });

    test('falls back to Bearer when no scheme supplied a challenge', () async {
      const extractor = FirstOf<String>([
        _Refuses(401, 'first'),
        _Refuses(401, 'last'),
      ]);

      final rejection = expectErr(await extractor.extract(request('GET', '/')));

      expect(rejection.status, 401);
      expect(rejection.challenge, 'Bearer');
    });

    test('stops at a 5xx rather than trying the rest', () async {
      // A server fault is not a reason to ask the caller for other
      // credentials; turning it into a 401 would hide a broken deployment.
      const extractor = FirstOf<String>([
        _Refuses(500, 'broken'),
        _Always('b'),
      ]);

      expectStatus(await extractor.extract(request('GET', '/')), 500);
    });

    test('refuses when it was given nothing to try', () async {
      const extractor = FirstOf<String>([]);

      expectStatus(await extractor.extract(request('GET', '/')), 401);
    });

    test('composes the real credential extractors', () async {
      const extractor = FirstOf<String>([
        BearerTokenExtractable(),
        ApiKeyExtractable(),
        SessionIdExtractable(),
      ]);
      final outcome = await extractor
          .extract(request('GET', '/', headers: const {'cookie': 'session=s'}));

      expect(expectOk(outcome), 's');
    });
  });

  group('from a request', () {
    test('reads an API key', () async {
      expect(
        await request('GET', '/', headers: const {'x-api-key': 'k'}).apiKey(),
        'k',
      );
    });

    test('reads a session id', () async {
      expect(
        await request('GET', '/', headers: const {'cookie': 'session=s'})
            .sessionId(),
        's',
      );
    });

    test('throws the 401 when neither is there', () async {
      expect(
        request('GET', '/').sessionId,
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 401)),
      );
    });
  });
}
