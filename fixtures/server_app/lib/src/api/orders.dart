import 'package:dust_server/server.dart';

import '../auth/caller.dart';
import '../auth/require_scope.dart';
import '../models/new_order.dart';
import '../models/order.dart';
import '../repo/order_store.dart';

part 'orders.g.dart';

// What an author writes. `orders.g.dart` beside it is what the server plugin
// will emit — hand-written until the plugin exists, and the spec it has to
// satisfy. The models it decodes are generated for real: see `models.g.dart`,
// which `dust build` produces from the annotations on `NewOrder` and `Order`.

/// `GET /` — an optional filter.
@GET('/', summary: 'List orders')
Future<List<Order>> listOrders(
  @Query('item') String? item,
  @State() OrderStore store,
) async {
  if (item == null) return store.orders;

  return store.orders.where((order) => order.item == item).toList();
}

/// `GET /{id}` — the `Err` carries its own status.
@GET('/{id}')
Future<Result<Order, Rejection>> readOrder(
  @Path() String id,
  @State() OrderStore store,
) async {
  final order = store.find(id);

  return order == null
      ? const Err(Rejection.notFound('no such order'))
      : Ok(order);
}

/// `POST /` — the body is validated by the generated `validate()`.
@POST('/', status: 201)
Future<Order> placeOrder(
  @Extract(OrdersWrite) Caller caller,
  @State() OrderStore store,
  @Body() NewOrder input,
) async {
  final order = Order(
    id: '${store.orders.length + 1}',
    item: input.item,
    quantity: input.quantity,
  );
  store.orders.add(order);

  return order;
}

/// `DELETE /{id}` — a `void` handler, so 204 and no body.
@DELETE('/{id}', status: 204)
Future<void> cancelOrder(
  @Extract(OrdersWrite) Caller caller,
  @Path() String id,
  @State() OrderStore store,
) async {
  store.orders.removeWhere((order) => order.id == id);
}
