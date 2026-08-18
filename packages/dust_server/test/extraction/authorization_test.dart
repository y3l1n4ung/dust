import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// The `Authorization` header is `<scheme> <credentials>`, and the scheme is
/// case-insensitive. An extractor that only matches `Bearer ` refuses a
/// conforming client that sent `bearer `, which is the kind of bug that shows
/// up once, from one SDK, in production.

void main() {
  group('basic credentials, at the edges', () {
    String header(String raw) => 'Basic ${base64.encode(utf8.encode(raw))}';

    Request sent(String value) => Request(
          'GET',
          Uri.parse('http://localhost/'),
          headers: {'authorization': value},
        );

    test('splits on the first colon, so a password may contain them', () async {
      // RFC 7617. Splitting on the last one, or refusing, locks out anyone
      // whose password has a colon in it — and they will never work out why.
      final outcome = await const BasicCredentialsExtractable()
          .extract(sent(header('ada:pa:ss:word')));

      final credentials = expectOk(outcome);
      expect(credentials.username, 'ada');
      expect(credentials.password, 'pa:ss:word');
    });

    test('accepts an empty password', () async {
      final credentials = expectOk(await const BasicCredentialsExtractable()
          .extract(sent(header('ada:'))));

      expect(credentials.username, 'ada');
      expect(credentials.password, isEmpty);
    });

    test('keeps a space in the password', () async {
      final credentials = expectOk(await const BasicCredentialsExtractable()
          .extract(sent(header('ada:s p'))));

      expect(credentials.password, 's p');
    });

    test('refuses a value with no colon at all', () async {
      expectStatus(
        await const BasicCredentialsExtractable()
            .extract(sent(header('nocolon'))),
        401,
      );
    });

    test('refuses something that is not base64', () async {
      expectStatus(
        await const BasicCredentialsExtractable()
            .extract(sent('Basic !!!not-base64!!!')),
        401,
      );
    });

    test('matches the scheme without regard to case', () async {
      // HTTP scheme names are case-insensitive, and clients do send `basic`.
      final credentials = expectOk(await const BasicCredentialsExtractable()
          .extract(sent('basic ${base64.encode(utf8.encode('ada:secret'))}')));

      expect(credentials.username, 'ada');
    });
  });

  Request withAuth(String value) =>
      request('GET', '/', headers: {'authorization': value});

  String basic(String user, String password) =>
      'Basic ${base64.encode(utf8.encode('$user:$password'))}';

  group('splitting the header', () {
    test('separates the scheme from the credentials', () {
      final header = Authorization.of(withAuth('Bearer abc'))!;

      expect([header.scheme, header.credentials], ['bearer', 'abc']);
    });

    test('lower-cases the scheme', () {
      expect(Authorization.of(withAuth('BEARER abc'))!.scheme, 'bearer');
    });

    test('matches a scheme case-insensitively', () {
      final header = Authorization.of(withAuth('bearer abc'))!;

      expect(header.isScheme('Bearer'), isTrue);
    });

    test('keeps credentials containing spaces', () {
      final header = Authorization.of(withAuth('Digest a=1, b=2'))!;

      expect(header.credentials, 'a=1, b=2');
    });

    test('trims the credentials', () {
      expect(Authorization.of(withAuth('Bearer   abc'))!.credentials, 'abc');
    });

    test('treats a bare word as a scheme with nothing after it', () {
      final header = Authorization.of(withAuth('Negotiate'))!;

      expect([header.scheme, header.credentials], ['negotiate', '']);
    });

    test('is null when the header is absent', () {
      expect(Authorization.of(request('GET', '/')), isNull);
    });
  });

  group('bearer tokens', () {
    test('come back without the scheme', () async {
      final outcome =
          await const BearerTokenExtractable().extract(withAuth('Bearer abc'));

      expect(expectOk(outcome), 'abc');
    });

    test('accept a lower-case scheme', () async {
      final outcome =
          await const BearerTokenExtractable().extract(withAuth('bearer abc'));

      expect(expectOk(outcome), 'abc');
    });

    test('reject another scheme with 401', () async {
      final outcome = await const BearerTokenExtractable()
          .extract(withAuth(basic('ada', 'secret')));

      expectStatus(outcome, 401);
    });

    test('reject an empty token', () async {
      final outcome =
          await const BearerTokenExtractable().extract(withAuth('Bearer '));

      expect(expectStatus(outcome, 401).message, 'the bearer token is empty');
    });

    test('carry the Bearer challenge', () async {
      final outcome =
          await const BearerTokenExtractable().extract(request('GET', '/'));

      expect(expectErr(outcome).challenge, 'Bearer');
    });

    test('carry a challenge naming a realm when asked', () async {
      const extractor = BearerTokenExtractable(challenge: 'Bearer realm="api"');
      final outcome = await extractor.extract(request('GET', '/'));

      expect(expectErr(outcome).challenge, 'Bearer realm="api"');
    });
  });

  group('basic credentials', () {
    test('decode a username and password', () async {
      final outcome = await const BasicCredentialsExtractable()
          .extract(withAuth(basic('ada', 'secret')));

      final given = expectOk(outcome);
      expect([given.username, given.password], ['ada', 'secret']);
    });

    test('split on the first colon only', () async {
      final outcome = await const BasicCredentialsExtractable()
          .extract(withAuth(basic('ada', 'se:cr:et')));

      expect(expectOk(outcome).password, 'se:cr:et');
    });

    test('reject a value that is not base64', () async {
      final outcome = await const BasicCredentialsExtractable()
          .extract(withAuth('Basic not-base64!!'));

      expectStatus(outcome, 401);
    });

    test('reject a decoded value with no colon', () async {
      final encoded = base64.encode(utf8.encode('nocolon'));
      final outcome = await const BasicCredentialsExtractable()
          .extract(withAuth('Basic $encoded'));

      expectStatus(outcome, 401);
    });

    test('reject an empty username', () async {
      final outcome = await const BasicCredentialsExtractable()
          .extract(withAuth(basic('', 'secret')));

      expectStatus(outcome, 401);
    });

    test('answer the Basic challenge, not the Bearer one', () async {
      final outcome = await const BasicCredentialsExtractable()
          .extract(request('GET', '/'));

      expect(
        expectErr(outcome).challenge,
        'Basic realm="restricted", charset="UTF-8"',
      );
    });

    test('name the realm they were given', () async {
      const extractor = BasicCredentialsExtractable(realm: 'admin');
      final outcome = await extractor.extract(request('GET', '/'));

      expect(expectErr(outcome).challenge, contains('realm="admin"'));
    });
  });

  group('from a request', () {
    test('reads a bearer token', () async {
      expect(await withAuth('Bearer abc').bearerToken(), 'abc');
    });

    test('reads basic credentials', () async {
      final given = await withAuth(basic('ada', 'secret')).basicCredentials();

      expect(given.username, 'ada');
    });

    test('throws the 401 when nothing was sent', () async {
      expect(
        request('GET', '/').bearerToken,
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 401)),
      );
    });
  });
}
