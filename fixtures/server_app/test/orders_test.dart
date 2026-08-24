import 'dart:convert';

import 'package:test/test.dart';

import 'support.dart';

/// The order routes, over a real SQLite database and real tokens.
///
/// Every query is scoped to the authenticated account in SQL, so the tests that
/// matter most are the ones checking one account cannot reach another's data.

void main() {
  group('listing', () {
    test('returns this account orders', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      await app.send('POST', '/orders',
          body: const {'item': 'shirt', 'quantity': 2}, token: token);

      final response = await app.send('GET', '/orders', token: token);

      expect(response.statusCode, 200);
      expect(jsonDecode(response.body), hasLength(1));
      expect(jsonDecode(response.body).first['item'], 'shirt');
    });

    test('does not return another account orders', () async {
      // The property the whole schema exists for.
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      await app.createAccount('bob@example.com', 'a different long password');
      final ada = await app.signIn('ada@example.com', 'correct horse battery');
      final bob =
          await app.signIn('bob@example.com', 'a different long password');

      await app.send('POST', '/orders',
          body: const {'item': 'ada shirt', 'quantity': 1}, token: ada);

      expect(jsonDecode((await app.send('GET', '/orders', token: bob)).body),
          isEmpty);
    });

    test('no token is 401', () async {
      final app = await testApp();

      expect((await app.send('GET', '/orders')).statusCode, 401);
    });

    test('a token that was never issued is 401', () async {
      final app = await testApp();

      expect(
        (await app.send('GET', '/orders', token: 'made-up')).statusCode,
        401,
      );
    });
  });

  group('reading one', () {
    test('reads your own', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      final placed = await app.send('POST', '/orders',
          body: const {'item': 'shirt', 'quantity': 2}, token: token);
      final id = jsonDecode(placed.body)['id'];

      final response = await app.send('GET', '/orders/$id', token: token);

      expect(response.statusCode, 200);
      expect(jsonDecode(response.body)['item'], 'shirt');
    });

    test('another account order is 404, not 403', () async {
      // 403 would confirm the order exists. A caller who cannot read it has no
      // business learning whether it is there.
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      await app.createAccount('bob@example.com', 'a different long password');
      final ada = await app.signIn('ada@example.com', 'correct horse battery');
      final bob =
          await app.signIn('bob@example.com', 'a different long password');

      final placed = await app.send('POST', '/orders',
          body: const {'item': 'ada shirt', 'quantity': 1}, token: ada);
      final id = jsonDecode(placed.body)['id'];

      final response = await app.send('GET', '/orders/$id', token: bob);

      expect(response.statusCode, 404);
      expect(jsonDecode(response.body)['error'], 'no such order');
    });

    test('an id that does not exist is the same 404', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      final response = await app.send('GET', '/orders/9999', token: token);

      expect(response.statusCode, 404);
    });

    test('a non-numeric id is 400 from the coercion', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      expect(
        (await app.send('GET', '/orders/abc', token: token)).statusCode,
        400,
      );
    });
  });

  group('placing', () {
    test('writes a row owned by the caller', () async {
      final app = await testApp();
      final accountId =
          await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      final response = await app.send('POST', '/orders',
          body: const {'item': 'shirt', 'quantity': 2}, token: token);

      expect(response.statusCode, 201);
      expect(jsonDecode(response.body)['accountId'], accountId);
    });

    test('the generated validator refuses a bad payload', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      final response = await app.send('POST', '/orders',
          body: const {'item': '', 'quantity': 99}, token: token);

      expect(response.statusCode, 422);
      expect(jsonDecode(response.body)['fields'], {
        'item': ['is required'],
        'quantity': ['must be 1 to 10'],
      });
    });

    test('a token without the scope is 403 and writes nothing', () async {
      final app = await testApp();
      await app.createAccount(
        'reader@example.com',
        'a read only long password',
        scopes: 'orders:read',
      );
      final token =
          await app.signIn('reader@example.com', 'a read only long password');

      final response = await app.send('POST', '/orders',
          body: const {'item': 'shirt', 'quantity': 1}, token: token);

      expect(response.statusCode, 403);
      expect(jsonDecode((await app.send('GET', '/orders', token: token)).body),
          isEmpty);
    });
  });

  group('cancelling', () {
    test('removes your own and answers 204', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      final placed = await app.send('POST', '/orders',
          body: const {'item': 'shirt', 'quantity': 1}, token: token);
      final id = jsonDecode(placed.body)['id'];

      final response = await app.send('DELETE', '/orders/$id', token: token);

      expect(response.statusCode, 204);
      expect(jsonDecode((await app.send('GET', '/orders', token: token)).body),
          isEmpty);
    });

    test('cannot cancel another account order', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      await app.createAccount('bob@example.com', 'a different long password');
      final ada = await app.signIn('ada@example.com', 'correct horse battery');
      final bob =
          await app.signIn('bob@example.com', 'a different long password');

      final placed = await app.send('POST', '/orders',
          body: const {'item': 'ada shirt', 'quantity': 1}, token: ada);
      final id = jsonDecode(placed.body)['id'];

      await app.send('DELETE', '/orders/$id', token: bob);

      expect(jsonDecode((await app.send('GET', '/orders', token: ada)).body),
          hasLength(1),
          reason: 'ada order survived');
    });
  });
}
