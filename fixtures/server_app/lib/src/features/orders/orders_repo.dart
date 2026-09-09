import 'package:dust_dart/db.dart';

import 'order_model.dart';

part 'orders_repo.g.dart';

/// Every query the orders feature makes.
@SqlxDao()
abstract final class OrdersRepo {
  /// Binds the queries to [db].
  const factory OrdersRepo(Executor db) = _$OrdersRepo;

  /// One page of an account's orders, newest first.
  ///
  /// Paged in SQL. Fetching everything and taking a slice in Dart is the same
  /// answer and an unbounded amount of work to produce it.
  @Query(r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE account_id = $1
ORDER BY id DESC
LIMIT $2 OFFSET $3
''')
  Future<Result<List<Order>, SqlxError>> pageFor(
    int accountId,
    int limit,
    int offset,
  );

  /// One order, scoped to its owner.
  @Query(r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE id = $1 AND account_id = $2
''')
  Future<Result<Order?, SqlxError>> orderFor(int id, int accountId);

  /// Places an order.
  @Query(r'''
INSERT INTO orders (account_id, item, quantity, placed_at)
VALUES ($1, $2, $3, $4)
''')
  Future<Result<ExecResult, SqlxError>> insertOrder(
    int accountId,
    String item,
    int quantity,
    String placedAt,
  );

  /// Cancels an order, scoped to its owner.
  @Query(r'''
DELETE FROM orders WHERE id = $1 AND account_id = $2
''')
  Future<Result<ExecResult, SqlxError>> deleteOrder(int id, int accountId);
}

/// One account's orders, narrowed by whichever filters were supplied.
///
/// Optional filters are separate queries, not one query built at run time.
/// Every branch below is a constant string `dust db build` described against
/// the schema, and the switch is exhaustive, so no combination reaches the
/// database unchecked. Building the `WHERE` clause by hand would take all four
/// outside validation to save three strings.
///
/// The SQL is written out at each call rather than pulled from a named
/// constant: validation reads the literal at the call site, so a `const` holding
/// the text is rejected as non-static. That is why these repeat themselves.
///
/// This stops scaling somewhere past three or four filters. At that point the
/// query genuinely is dynamic and belongs on the facade's `unsafe` hatch, where
/// it is visible as such.
Future<Result<List<Order>, SqlxError>> searchOrders(
  Executor db, {
  required int accountId,
  String? item,
  int? minQuantity,
}) {
  final query = switch ((item, minQuantity)) {
    (null, null) => queryAs<Order>(
        r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE account_id = $1
ORDER BY id DESC
''',
        [accountId],
      ),
    (final item?, null) => queryAs<Order>(
        r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE account_id = $1 AND item = $2
ORDER BY id DESC
''',
        [accountId, item],
      ),
    (null, final minQuantity?) => queryAs<Order>(
        r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE account_id = $1 AND quantity >= $2
ORDER BY id DESC
''',
        [accountId, minQuantity],
      ),
    (final item?, final minQuantity?) => queryAs<Order>(
        r'''
SELECT id, account_id, item, quantity, placed_at FROM orders
WHERE account_id = $1 AND item = $2 AND quantity >= $3
ORDER BY id DESC
''',
        [accountId, item, minQuantity],
      ),
  };
  return query.fetchAll(db);
}
