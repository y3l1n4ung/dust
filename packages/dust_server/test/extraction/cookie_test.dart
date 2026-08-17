import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// A browser sends every cookie for the origin in one header, so reading one
/// means parsing a list written by code that is not yours. What matters is
/// that a stray cookie from something else on the domain cannot break the
/// request, and that the one wanted is still found.

void main() {
  Request withCookies(String header) =>
      request('GET', '/', headers: {'cookie': header});

  group('parsing the header', () {
    test('reads a single pair', () {
      expect(CookieJar.of(withCookies('session=abc'))['session'], 'abc');
    });

    test('reads one out of several', () {
      final jar = CookieJar.of(withCookies('theme=dark; session=abc; a=1'));

      expect(jar['session'], 'abc');
    });

    test('ignores the spacing around pairs', () {
      final jar = CookieJar.of(withCookies('  theme=dark ;session=abc  '));

      expect([jar['theme'], jar['session']], ['dark', 'abc']);
    });

    test('unwraps a quoted value', () {
      expect(CookieJar.of(withCookies('session="abc"'))['session'], 'abc');
    });

    test('keeps a value containing an equals sign', () {
      expect(CookieJar.of(withCookies('t=a=b'))['t'], 'a=b');
    });

    test('keeps an empty value', () {
      expect(CookieJar.of(withCookies('session='))['session'], '');
    });

    test('skips a pair with no name', () {
      final jar = CookieJar.of(withCookies('=orphan; session=abc'));

      expect(jar.names, ['session']);
    });

    test('skips a bare token with no equals sign', () {
      final jar = CookieJar.of(withCookies('broken; session=abc'));

      expect(jar.names, ['session']);
    });

    test('keeps the first of a repeated name', () {
      expect(CookieJar.of(withCookies('a=1; a=2'))['a'], '1');
    });

    test('is empty when no header was sent', () {
      expect(CookieJar.of(request('GET', '/')).names, isEmpty);
    });

    test('treats names as case-sensitive, as cookies are', () {
      final jar = CookieJar.of(withCookies('Session=abc'));

      expect(jar['session'], isNull);
      expect(jar['Session'], 'abc');
    });

    test('reports what it holds', () {
      final jar = CookieJar.of(withCookies('a=1; b=2'));

      expect(jar.contains('a'), isTrue);
      expect(jar.contains('c'), isFalse);
    });
  });

  group('the extractor', () {
    test('reads a cookie', () async {
      final outcome = await const CookieExtractable<String>('session')
          .extract(withCookies('session=abc'));

      expect(expectOk(outcome), 'abc');
    });

    test('coerces to the type asked for', () async {
      final outcome = await const CookieExtractable<int>('count')
          .extract(withCookies('count=7'));

      expect(expectOk(outcome), 7);
    });

    test('rejects with 400 when a required cookie is missing', () async {
      final outcome = await const CookieExtractable<String>('session')
          .extract(request('GET', '/'));

      expectStatus(outcome, 400);
    });

    test('gives null when a nullable cookie is missing', () async {
      final outcome = await const CookieExtractable<String?>('session')
          .extract(request('GET', '/'));

      expect(expectOk(outcome), isNull);
    });

    test('rejects with 400 when the value will not coerce', () async {
      final outcome = await const CookieExtractable<int>('count')
          .extract(withCookies('count=many'));

      expectStatus(outcome, 400);
    });

    test('reads the whole jar', () async {
      final outcome =
          await const CookieJarExtractable().extract(withCookies('a=1; b=2'));

      expect(expectOk(outcome).names, ['a', 'b']);
    });
  });

  group('from a request', () {
    test('reads one cookie', () async {
      expect(await withCookies('session=abc').cookie<String>('session'), 'abc');
    });

    test('reads the jar', () async {
      expect((await withCookies('a=1').cookies())['a'], '1');
    });

    test('throws when a required cookie is missing', () async {
      expect(
        () => request('GET', '/').cookie<String>('session'),
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 400)),
      );
    });
  });
}
