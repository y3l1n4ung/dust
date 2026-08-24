import 'package:dust_dart/db.dart';

import 'order_model.dart';

part 'orders_repo.g.dart';

/// Every query the orders feature makes.
@SqlxDao()
abstract final class OrdersRepo {
  /// Binds the queries to [db].
  const factory OrdersRepo(DatabaseExecutor db) = _$OrdersRepo;

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
