import 'dart:convert';

import 'package:dust_dart/db.dart';
import 'package:server_app/server_app.dart';
import 'package:test/test.dart';

import 'support.dart';

/// Signing in, and the properties that make it real auth rather than a demo.

void main() {
  group('passwords', () {
    test('the same password with different salts hashes differently', () {
      // Per-account salts, so one rainbow table cannot cover two accounts and
      // two people with the same password do not share a hash.
      final first =
          Passwords.hash('correct horse battery', Passwords.newSalt());
      final second =
          Passwords.hash('correct horse battery', Passwords.newSalt());

      expect(first, isNot(second));
    });

    test('the same password and salt hash the same, so verify works', () {
      final salt = Passwords.newSalt();
      final hash = Passwords.hash('correct horse battery', salt);

      expect(Passwords.verify('correct horse battery', salt, hash), isTrue);
      expect(Passwords.verify('wrong', salt, hash), isFalse);
    });

    test('a salt is not reused', () {
      final salts = {for (var i = 0; i < 20; i++) Passwords.newSalt()};

      expect(salts, hasLength(20));
    });

    test('the comparison is length-safe', () {
      expect(Passwords.constantTimeEquals('abc', 'abcd'), isFalse);
      expect(Passwords.constantTimeEquals('abc', 'abc'), isTrue);
    });
  });

  group('tokens', () {
    test('two issued tokens never collide', () {
      final issued = {for (var i = 0; i < 50; i++) Tokens.issue()};

      expect(issued, hasLength(50));
    });

    test('a token is long enough not to be guessed', () {
      expect(Tokens.issue().length, greaterThanOrEqualTo(32));
    });

    test('the fingerprint is stable and is not the token', () {
      final token = Tokens.issue();

      expect(Tokens.fingerprint(token), Tokens.fingerprint(token));
      expect(Tokens.fingerprint(token), isNot(contains(token)));
    });
  });

  group('POST /auth/tokens', () {
    test('issues a token for the right password', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');

      final response = await app.send(
        'POST',
        '/auth/tokens',
        body: const {
          'email': 'ada@example.com',
          'password': 'correct horse battery',
        },
      );

      expect(response.statusCode, 201);
      expect(jsonDecode(response.body)['token'], isA<String>());
      expect(jsonDecode(response.body)['expiresAt'], isA<String>());
    });

    test('stores the fingerprint, never the token', () async {
      // A database read must not hand over working credentials. Proven with the
      // generated lookup: the fingerprint finds the account, the token does not.
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');
      final now = DateTime.now().toUtc().toIso8601String();

      final byFingerprint =
          await app.accounts.accountForToken(Tokens.fingerprint(token), now);
      final byToken = await app.accounts.accountForToken(token, now);

      expect((byFingerprint as Ok<Account?, SqlxError>).value, isNotNull);
      expect(
        (byToken as Ok<Account?, SqlxError>).value,
        isNull,
        reason: 'the raw token is not what is stored',
      );
    });

    test('an expired token stops working', () async {
      // Expiry is checked in SQL, so a row that is past its date cannot be used
      // by code that forgets to look.
      final app = await testApp();
      final accountId =
          await app.createAccount('ada@example.com', 'correct horse battery');
      final token = Tokens.issue();
      await app.accounts.insertToken(
        accountId,
        Tokens.fingerprint(token),
        DateTime.now()
            .toUtc()
            .subtract(const Duration(days: 1))
            .toIso8601String(),
      );

      final response = await app.send('GET', '/orders', token: token);

      expect(response.statusCode, 401);
    });

    test('a wrong password is refused', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');

      final response = await app.send(
        'POST',
        '/auth/tokens',
        body: const {'email': 'ada@example.com', 'password': 'wrong password'},
      );

      expect(response.statusCode, 401);
    });

    test('an unknown email answers exactly like a wrong password', () async {
      // Different messages here turn the endpoint into a way to discover which
      // email addresses have accounts.
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');

      final wrongPassword = await app.send(
        'POST',
        '/auth/tokens',
        body: const {'email': 'ada@example.com', 'password': 'wrong password'},
      );
      final unknownEmail = await app.send(
        'POST',
        '/auth/tokens',
        body: const {
          'email': 'nobody@example.com',
          'password': 'wrong password'
        },
      );

      expect(unknownEmail.statusCode, wrongPassword.statusCode);
      expect(unknownEmail.body, wrongPassword.body);
    });

    test('a short password is refused by the validator, not the database',
        () async {
      final app = await testApp();

      final response = await app.send(
        'POST',
        '/auth/tokens',
        body: const {'email': 'ada@example.com', 'password': 'short'},
      );

      expect(response.statusCode, 422);
      expect(jsonDecode(response.body)['fields'], {
        'password': ['must be at least 12 characters'],
      });
    });

    test('a malformed email is refused', () async {
      final app = await testApp();

      final response = await app.send(
        'POST',
        '/auth/tokens',
        body: const {
          'email': 'not-an-email',
          'password': 'correct horse battery'
        },
      );

      expect(response.statusCode, 422);
    });
  });
}
