import 'package:dust_dart/db.dart';

import 'package:postgres_app/src/order.dart';

part 'orders_repo.g.dart';

/// Every query the fixture makes.
///
/// The SQL is written the way it would be on either dialect. What differs is
/// underneath: PostgreSQL reads `$1` itself, and a list binds as an array so
/// `ANY($1)` needs no encoding where SQLite reaches the same place through
/// `json_each`.
@SqlxDao()
abstract final class OrdersRepo {
  const factory OrdersRepo(Executor db) = _$OrdersRepo;

  /// One account's orders, oldest first.
  @Query(r'''
SELECT id, account_id, item, quantity, express, placed_at FROM orders
WHERE account_id = $1
ORDER BY id
''')
  Future<Result<List<Order>, SqlxError>> forAccount(int accountId);

  /// Several orders by id, as one bound array.
  @Query(r'''
SELECT id, account_id, item, quantity, express, placed_at FROM orders
WHERE id = ANY($1)
ORDER BY id
''')
  Future<Result<List<Order>, SqlxError>> byIds(List<int> ids);

  /// A repeated placeholder, which binds once here and twice on SQLite.
  @Query(r'''
SELECT id, account_id, item, quantity, express, placed_at FROM orders
WHERE account_id = $1 AND (quantity >= $2 OR $2 = 0)
ORDER BY id
''')
  Future<Result<List<Order>, SqlxError>> atLeast(int accountId, int quantity);

  /// PostgreSQL has no `lastInsertId`, so the new row is asked for.
  @Query(r'''
INSERT INTO orders (account_id, item, quantity, express)
VALUES ($1, $2, $3, $4)
RETURNING id, account_id, item, quantity, express, placed_at
''')
  Future<Result<Order, SqlxError>> place(
    int accountId,
    String item,
    int quantity,
    // Positional to match the column order the query binds.
    // ignore: avoid_positional_boolean_parameters
    bool express,
  );

  /// How many orders an account has placed.
  @Query(r'SELECT count(*) FROM orders WHERE account_id = $1')
  Future<Result<int, SqlxError>> countFor(int accountId);
}
