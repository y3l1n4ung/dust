import 'package:dust_server/server.dart';

import '../accounts/account_model.dart';
import '../accounts/require_scope.dart';
import 'new_order_dto.dart';
import 'order_model.dart';
import 'orders_repo.dart';

part 'orders_handler.g.dart';

// Every query is scoped to the authenticated account, in SQL. Filtering in Dart
// after fetching everything is how one customer reads another's orders on the
// day somebody forgets an `if`.

/// `GET /?limit=&offset=` — this account's orders, paged.
///
/// The limit is capped rather than trusted: `?limit=1000000` is a denial-of-
/// service request dressed as pagination.
@GET('/', summary: 'List your orders')
Future<Result<List<Order>, Rejection>> listOrders(
  @Extract(RequireScope) Account account,
  @Query('limit') int? limit,
  @Query('offset') int? offset,
  @State() OrdersRepo repo,
) async {
  final page = (limit ?? 20).clamp(1, 100);
  final from = (offset ?? 0).clamp(0, 1 << 30);

  return switch (await repo.pageFor(account.id, page, from)) {
    Ok(:final value) => Ok(value),
    Err(:final error) => Err(_internal(error)),
  };
}

/// `GET /{id}` — one order, if it belongs to this account.
///
/// A miss and someone else's order answer the same 404. Distinguishing them
/// would confirm an order with that id exists.
@GET('/{id}')
Future<Result<Order, Rejection>> readOrder(
  @Path() int id,
  @Extract(RequireScope) Account account,
  @State() OrdersRepo repo,
) async {
  return switch (await repo.orderFor(id, account.id)) {
    Ok(value: final order?) => Ok(order),
    Ok() => const Err(Rejection.notFound('no such order')),
    Err(:final error) => Err(_internal(error)),
  };
}

/// `POST /` — place one.
@POST('/', status: 201)
Future<Result<Order, Rejection>> placeOrder(
  @Extract(OrdersWrite) Account account,
  @State() OrdersRepo repo,
  @Body() NewOrder input,
) async {
  final placedAt = DateTime.now().toUtc().toIso8601String();
  final inserted = await repo.insertOrder(
    account.id,
    input.item,
    input.quantity,
    placedAt,
  );

  return switch (inserted) {
    Err(:final error) => Err(_internal(error)),
    Ok(:final value) => Ok(
        Order(
          id: value.lastInsertId!,
          accountId: account.id,
          item: input.item,
          quantity: input.quantity,
          placedAt: placedAt,
        ),
      ),
  };
}

/// `DELETE /{id}` — cancel one, if it belongs to this account.
@DELETE('/{id}', status: 204)
Future<Result<void, Rejection>> cancelOrder(
  @Extract(OrdersWrite) Account account,
  @Path() int id,
  @State() OrdersRepo repo,
) async {
  return switch (await repo.deleteOrder(id, account.id)) {
    Err(:final error) => Err(_internal(error)),
    Ok() => const Ok(null),
  };
}

/// Answers for a database fault without telling the client what broke.
Rejection _internal(Object error) {
  ServerErrors.report(error, StackTrace.current);

  return const Rejection.internal();
}
