import 'package:dust_dart/db.dart';
import 'package:dust_server/server.dart';

import '../../shared/db/database.dart';
import '../accounts/account_model.dart';
import '../accounts/require_scope.dart';
import '../orders/new_order_dto.dart';
import '../orders/order_model.dart';
import '../orders/orders_repo.dart';
import 'inventory_repo.dart';
import 'stock_model.dart';

part 'inventory_router.g.dart';

// The feature that needs a transaction: reserving stock and writing the order
// are one unit of work.

/// `GET /stock` — what is left.
@GET('/stock', summary: 'Remaining stock')
Future<Result<List<Stock>, Rejection>> listStock(
  @Extract(RequireScope) Account account,
  @State() InventoryRepo repo,
) async {
  return switch (await repo.allStock()) {
    Ok(:final value) => Ok(value),
    Err() => const Err(Rejection.internal()),
  };
}

/// `POST /checkout` — reserve stock and place the order together.
///
/// The order of the two writes is the whole design. Stock is reserved **first**,
/// so a sold-out item fails before an order exists. Insert the order first and
/// you are left with a paid order you cannot ship.
///
/// A 409 rather than a 422: "somebody bought the last one" is an ordinary
/// outcome of a shop, not a malformed request.
@POST('/checkout', status: 201)
Future<Result<Order, Rejection>> checkout(
  @Extract(OrdersWrite) Account account,
  @State() AppDatabase database,
  @Body() NewOrder input,
) async {
  final placedAt = DateTime.now().toUtc().toIso8601String();

  final outcome = await database.transaction<Result<Order, Rejection>>(
    (tx) async {
      final inventory = InventoryRepo(tx);
      final orders = OrdersRepo(tx);

      final reserved = await inventory.reserve(input.quantity, input.item);
      if (reserved case Err(:final error)) return Err(error);
      if ((reserved as Ok<ExecResult, SqlxError>).value.rowsAffected == 0) {
        // Nothing matched: either the item does not exist or there is not
        // enough. Both mean the same thing to a customer.
        return const Ok(
          Err(Rejection.conflict('not enough stock left')),
        );
      }

      final inserted = await orders.insertOrder(
        account.id,
        input.item,
        input.quantity,
        placedAt,
      );
      if (inserted case Err(:final error)) return Err(error);

      return Ok(
        Ok(
          Order(
            id: (inserted as Ok<ExecResult, SqlxError>).value.lastInsertId!,
            accountId: account.id,
            item: input.item,
            quantity: input.quantity,
            placedAt: placedAt,
          ),
        ),
      );
    },
  );

  return switch (outcome) {
    Ok(:final value) => value,
    Err() => const Err(Rejection.internal()),
  };
}
