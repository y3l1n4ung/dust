import 'package:server_app/src/features/orders/orders_repo.dart';
import 'package:test/test.dart';

import 'testing.dart';

/// The two patterns that exist so nobody has to build SQL by hand.
///
/// Both are described at build time, which is the whole claim: an `IN` list and
/// a set of optional filters are the two cases that used to send people to the
/// unchecked path, and neither needs it.

void main() {
  group('an IN list is one bound list', () {
    test('selects only the named items', () async {
      final app = await testApp();
      await app.inventory.addStock('shirt', 5);
      await app.inventory.addStock('socks', 3);
      await app.inventory.addStock('hat', 1);

      final stock = await app.inventory.stockForItems(['shirt', 'hat']);

      expect(
        stock.match(
          ok: (rows) => rows.map((row) => row.item).toList(),
          err: (_) => <String>[],
        ),
        <String>['hat', 'shirt'],
      );
    });

    test('an empty list selects nothing rather than failing', () async {
      final app = await testApp();
      await app.inventory.addStock('shirt', 5);

      final stock = await app.inventory.stockForItems(const []);

      expect(stock.match(ok: (rows) => rows.length, err: (_) => -1), 0);
    });
  });

  group('optional filters are separate queries', () {
    Future<TestApp> shop() async {
      final app = await testApp();
      await app.createAccount('ada@example.com', 'correct horse battery');
      await app.inventory.addStock('shirt', 10);
      await app.inventory.addStock('socks', 10);
      final token =
          await app.signIn('ada@example.com', 'correct horse battery');
      for (final order in const [
        ('shirt', 1),
        ('shirt', 4),
        ('socks', 2),
      ]) {
        await (app.client.post('/inventory/checkout')
              ..bearer(token)
              ..json({'item': order.$1, 'quantity': order.$2}))
            .send();
      }
      return app;
    }

    test('every combination of filters returns the right orders', () async {
      final app = await shop();
      final accountId = 1;

      Future<List<String>> search({String? item, int? minQuantity}) async {
        final result = await searchOrders(
          app.database.connection,
          accountId: accountId,
          item: item,
          minQuantity: minQuantity,
        );
        return result.match(
          ok: (orders) =>
              orders.map((order) => '${order.item}:${order.quantity}').toList()
                ..sort(),
          err: (error) => <String>['err:$error'],
        );
      }

      expect(await search(), <String>['shirt:1', 'shirt:4', 'socks:2']);
      expect(await search(item: 'shirt'), <String>['shirt:1', 'shirt:4']);
      expect(await search(minQuantity: 2), <String>['shirt:4', 'socks:2']);
      expect(
        await search(item: 'shirt', minQuantity: 2),
        <String>['shirt:4'],
      );
    });
  });
}
