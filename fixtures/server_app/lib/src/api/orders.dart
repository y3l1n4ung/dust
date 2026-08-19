import 'package:dust_server/server.dart';

import '../auth/require_scope.dart';
import '../db/queries.dart';
import '../models/account.dart';
import '../models/new_order.dart';
import '../models/order.dart';

part 'orders.g.dart';

// What an author writes. `orders.g.dart` is what the server plugin will emit.
//
// Every query is scoped to the authenticated account, in SQL. Filtering in
// Dart after fetching everything is how one customer reads another's orders on
// the day somebody forgets an `if`.

/// `GET /` — this account's orders.
@GET('/', summary: 'List your orders')
Future<Result<List<Order>, Rejection>> listOrders(
  @Extract(RequireScope) Account account,
  @State() AppQueries queries,
) async {
  return switch (await queries.ordersFor(account.id)) {
    Ok(:final value) => Ok(value),
    Err(:final error) => Err(_reportAsInternal(error)),
  };
}

/// `GET /{id}` — one order, if it belongs to this account.
///
/// A miss and someone else's order answer the same 404. Distinguishing them
/// would confirm that an order with that id exists, which is not this caller's
/// business.
@GET('/{id}')
Future<Result<Order, Rejection>> readOrder(
  @Path() int id,
  @Extract(RequireScope) Account account,
  @State() AppQueries queries,
) async {
  return switch (await queries.orderFor(id, account.id)) {
    Ok(value: final order?) => Ok(order),
    Ok() => const Err(Rejection.notFound('no such order')),
    Err(:final error) => Err(_reportAsInternal(error)),
  };
}

/// `POST /` — place one.
@POST('/', status: 201)
Future<Result<Order, Rejection>> placeOrder(
  @Extract(OrdersWrite) Account account,
  @State() AppQueries queries,
  @Body() NewOrder input,
) async {
  final placedAt = DateTime.now().toUtc().toIso8601String();
  final inserted = await queries.insertOrder(
    account.id,
    input.item,
    input.quantity,
    placedAt,
  );

  return switch (inserted) {
    Err(:final error) => Err(_reportAsInternal(error)),
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
  @State() AppQueries queries,
) async {
  return switch (await queries.deleteOrder(id, account.id)) {
    Err(:final error) => Err(_reportAsInternal(error)),
    Ok() => const Ok(null),
  };
}

/// Answers for a database fault without telling the client what broke.
Rejection _reportAsInternal(Object error) {
  ServerErrors.report(error, StackTrace.current);

  return const Rejection.internal();
}
