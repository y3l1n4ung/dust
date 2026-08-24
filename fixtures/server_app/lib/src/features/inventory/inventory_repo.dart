import 'package:dust_dart/db.dart';

import 'stock_model.dart';

part 'inventory_repo.g.dart';

/// Every query the inventory feature makes.
@SqlxDao()
abstract final class InventoryRepo {
  /// Binds the queries to [db], a connection **or a transaction**.
  ///
  /// That parameter is why a repo and a database are different types: reserving
  /// stock and writing an order are one unit of work, and half of it is worse
  /// than neither.
  const factory InventoryRepo(DatabaseExecutor db) = _$InventoryRepo;

  /// What is left of one item.
  @Query(r'SELECT item, on_hand FROM stock WHERE item = $1')
  Future<Result<Stock?, SqlxError>> stockFor(String item);

  /// Everything in stock.
  @Query(r'SELECT item, on_hand FROM stock ORDER BY item')
  Future<Result<List<Stock>, SqlxError>> allStock();

  /// Adds stock, creating the row when it is the first of its kind.
  @Query(r'''
INSERT INTO stock (item, on_hand) VALUES ($1, $2)
ON CONFLICT (item) DO UPDATE SET on_hand = on_hand + $2
''')
  Future<Result<ExecResult, SqlxError>> addStock(String item, int quantity);

  /// Takes [quantity] off the shelf, if there is that much left.
  ///
  /// One statement checks **and** subtracts. The obvious version — read the
  /// count, and if it is enough write the new one — has a window between the
  /// read and the write where two requests both see the last item and both take
  /// it. Putting the condition in the `WHERE` closes the window: the loser
  /// matches zero rows.
  @Query(r'''
UPDATE stock SET on_hand = on_hand - $1
WHERE item = $2 AND on_hand >= $1
''')
  Future<Result<ExecResult, SqlxError>> reserve(int quantity, String item);
}
