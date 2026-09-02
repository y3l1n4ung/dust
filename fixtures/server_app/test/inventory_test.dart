import 'package:test/test.dart';

import 'testing.dart';

/// Checkout, which is the feature that needs a transaction.
///
/// Everything else in this fixture is shape you can read off the code. Whether
/// it oversells is a property of the order two writes happen in, and only a
/// race can tell you.

void main() {
  Future<({TestApp app, String token})> shopWith(int onHand) async {
    final app = await testApp();
    await app.createAccount('ada@example.com', 'correct horse battery');
    await app.inventory.addStock('shirt', onHand);

    return (
      app: app,
      token: await app.signIn('ada@example.com', 'correct horse battery'),
    );
  }

  group('checkout', () {
    test('places the order and takes the stock', () async {
      final shop = await shopWith(5);

      final response = await (shop.app.client.post('/inventory/checkout')
              ..bearer(shop.token)
              ..json(const {'item': 'shirt', 'quantity': 2}))
          .send();

      response.assertCreated();
      expect((response.json as Map)['quantity'], 2);

      final stock = await (shop.app.client.get('/inventory/stock')
              ..bearer(shop.token))
          .send();
      expect((stock.json as List).first['onHand'], 3);
    });

    test('refuses more than is left, with 409', () async {
      // Not a 422: the payload is fine, and "somebody bought the last one" is
      // an ordinary outcome of a shop rather than a malformed request.
      final shop = await shopWith(1);

      final response = await (shop.app.client.post('/inventory/checkout')
              ..bearer(shop.token)
              ..json(const {'item': 'shirt', 'quantity': 5}))
          .send();

      response.assertConflict();
      expect((response.json as Map)['error'], 'not enough stock left');
    });

    test('a refused checkout writes no order', () async {
      // The transaction is the point: an order row for a refused checkout means
      // the insert escaped the rollback.
      final shop = await shopWith(1);

      await (shop.app.client.post('/inventory/checkout')
              ..bearer(shop.token)
              ..json(const {'item': 'shirt', 'quantity': 5}))
          .send();

      expect(
        (await (shop.app.client.get('/orders')..bearer(shop.token)).send())
            .json,
        isEmpty,
      );
    });

    test('a refused checkout leaves the stock alone', () async {
      final shop = await shopWith(1);

      await (shop.app.client.post('/inventory/checkout')
              ..bearer(shop.token)
              ..json(const {'item': 'shirt', 'quantity': 5}))
          .send();

      final stock = await (shop.app.client.get('/inventory/stock')
              ..bearer(shop.token))
          .send();
      expect((stock.json as List).first['onHand'], 1);
    });

    test('an unknown item is refused the same way', () async {
      final shop = await shopWith(5);

      (await (shop.app.client.post('/inventory/checkout')
                  ..bearer(shop.token)
                  ..json(const {'item': 'hat', 'quantity': 1}))
              .send())
          .assertConflict();
    });

    test('two simultaneous buyers of the last one: exactly one wins', () async {
      // The property the conditional UPDATE exists for.
      final shop = await shopWith(1);

      final attempts = await Future.wait([
        (shop.app.client.post('/inventory/checkout')
                ..bearer(shop.token)
                ..json(const {'item': 'shirt', 'quantity': 1}))
            .send(),
        (shop.app.client.post('/inventory/checkout')
                ..bearer(shop.token)
                ..json(const {'item': 'shirt', 'quantity': 1}))
            .send(),
      ]);

      final codes = attempts.map((r) => r.statusCode).toList();
      expect(codes, containsAll([201, 409]));

      final stock = await (shop.app.client.get('/inventory/stock')
              ..bearer(shop.token))
          .send();
      expect((stock.json as List).first['onHand'], 0);
    });

    test('ten buyers of five: five win, and stock lands on zero', () async {
      final shop = await shopWith(5);

      final attempts = await Future.wait([
        for (var index = 0; index < 10; index++)
          (shop.app.client.post('/inventory/checkout')
                  ..bearer(shop.token)
                  ..json(const {'item': 'shirt', 'quantity': 1}))
              .send(),
      ]);

      expect(attempts.where((r) => r.statusCode == 201).length, 5);
      expect(attempts.where((r) => r.statusCode == 409).length, 5);

      final stock = await (shop.app.client.get('/inventory/stock')
              ..bearer(shop.token))
          .send();
      expect((stock.json as List).first['onHand'], 0);
    });

    test('every success has an order row and every refusal has none', () async {
      final shop = await shopWith(5);

      final attempts = await Future.wait([
        for (var index = 0; index < 10; index++)
          (shop.app.client.post('/inventory/checkout')
                  ..bearer(shop.token)
                  ..json(const {'item': 'shirt', 'quantity': 1}))
              .send(),
      ]);
      final placed = attempts.where((r) => r.statusCode == 201).length;

      expect(
        (await (shop.app.client.get('/orders')..bearer(shop.token)).send())
            .json,
        hasLength(placed),
      );
    });

    test('stock never goes negative, whatever the arrival order', () async {
      final shop = await shopWith(5);

      await Future.wait([
        for (var index = 0; index < 20; index++)
          (shop.app.client.post('/inventory/checkout')
                  ..bearer(shop.token)
                  ..json({
                    'item': 'shirt',
                    'quantity': index.isEven ? 1 : 2,
                  }))
              .send(),
      ]);

      final stock = await (shop.app.client.get('/inventory/stock')
              ..bearer(shop.token))
          .send();
      expect((stock.json as List).first['onHand'], greaterThanOrEqualTo(0));
    });
  });
}
