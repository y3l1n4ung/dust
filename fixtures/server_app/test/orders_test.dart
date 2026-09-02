import 'package:test/test.dart';

import 'testing.dart';

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

      await (app.client.post('/orders')
              ..bearer(token)
              ..json(const {'item': 'shirt', 'quantity': 2}))
          .send();

      final response =
          await (app.client.get('/orders')..bearer(token)).send();
      final orders = response.json as List;

      response.assertOk();
      expect(orders, hasLength(1));
      expect(orders.first['item'], 'shirt');
    });

    test('does not return another account orders', () async {
      // The property the whole schema exists for.
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      await app.createAccount('bob@example.com', 'a different long password');
      final ada = await app.signIn('ada@example.com', 'correct horse battery');
      final bob =
          await app.signIn('bob@example.com', 'a different long password');

      await (app.client.post('/orders')
              ..bearer(ada)
              ..json(const {'item': 'ada shirt', 'quantity': 1}))
          .send();

      expect(
        (await (app.client.get('/orders')..bearer(bob)).send()).json,
        isEmpty,
      );
    });

    test('no token is 401', () async {
      final app = await testApp();

      (await app.client.get('/orders').send()).assertUnauthorized();
    });

    test('a token that was never issued is 401', () async {
      final app = await testApp();

      (await (app.client.get('/orders')..bearer('made-up')).send())
          .assertUnauthorized();
    });
  });

  group('reading one', () {
    test('reads your own', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      final placed = await (app.client.post('/orders')
              ..bearer(token)
              ..json(const {'item': 'shirt', 'quantity': 2}))
          .send();
      final id = (placed.json as Map)['id'];

      final response =
          await (app.client.get('/orders/$id')..bearer(token)).send();

      response.assertOk();
      expect((response.json as Map)['item'], 'shirt');
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

      final placed = await (app.client.post('/orders')
              ..bearer(ada)
              ..json(const {'item': 'ada shirt', 'quantity': 1}))
          .send();
      final id = (placed.json as Map)['id'];

      final response =
          await (app.client.get('/orders/$id')..bearer(bob)).send();

      response.assertNotFound();
      expect((response.json as Map)['error'], 'no such order');
    });

    test('an id that does not exist is the same 404', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      (await (app.client.get('/orders/9999')..bearer(token)).send())
          .assertNotFound();
    });

    test('a non-numeric id is 400 from the coercion', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      (await (app.client.get('/orders/abc')..bearer(token)).send())
          .assertBadRequest();
    });
  });

  group('placing', () {
    test('writes a row owned by the caller', () async {
      final app = await testApp();
      final accountId =
          await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      final response = await (app.client.post('/orders')
              ..bearer(token)
              ..json(const {'item': 'shirt', 'quantity': 2}))
          .send();

      response.assertCreated();
      expect((response.json as Map)['accountId'], accountId);
    });

    test('the generated validator refuses a bad payload', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      final response = await (app.client.post('/orders')
              ..bearer(token)
              ..json(const {'item': '', 'quantity': 99}))
          .send();

      response.assertUnprocessable();
      expect((response.json as Map)['fields'], {
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

      (await (app.client.post('/orders')
                  ..bearer(token)
                  ..json(const {'item': 'shirt', 'quantity': 1}))
              .send())
          .assertForbidden();

      expect(
        (await (app.client.get('/orders')..bearer(token)).send()).json,
        isEmpty,
      );
    });
  });

  group('cancelling', () {
    test('removes your own and answers 204', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');

      final placed = await (app.client.post('/orders')
              ..bearer(token)
              ..json(const {'item': 'shirt', 'quantity': 1}))
          .send();
      final id = (placed.json as Map)['id'];

      (await (app.client.delete('/orders/$id')..bearer(token)).send())
          .assertNoContent();

      expect(
        (await (app.client.get('/orders')..bearer(token)).send()).json,
        isEmpty,
      );
    });

    test('cannot cancel another account order', () async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      await app.createAccount('bob@example.com', 'a different long password');
      final ada = await app.signIn('ada@example.com', 'correct horse battery');
      final bob =
          await app.signIn('bob@example.com', 'a different long password');

      final placed = await (app.client.post('/orders')
              ..bearer(ada)
              ..json(const {'item': 'ada shirt', 'quantity': 1}))
          .send();
      final id = (placed.json as Map)['id'];

      await (app.client.delete('/orders/$id')..bearer(bob)).send();

      expect(
        (await (app.client.get('/orders')..bearer(ada)).send()).json,
        hasLength(1),
        reason: 'ada order survived',
      );
    });
  });
}
