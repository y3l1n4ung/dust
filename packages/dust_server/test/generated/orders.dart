import 'package:dust_dart/serde.dart';
import 'package:dust_server/server.dart';

import 'models.dart';

// A second worked example, exercising more of the surface than
// `functions.dart` does: an optional query, a validated body, a custom
// extractor carrying configuration, a `Result` return, a 201, and a 204.
//
// This file is what an author writes. `orders.g.dart` beside it is what the
// plugin emits, hand-written for now — and is applied as a part, which is how
// every Dust generator delivers its output.

part 'orders.g.dart';

/// What `POST /orders` accepts.
final class NewOrder implements Validatable {
  const NewOrder({required this.item, required this.quantity});

  static NewOrder deserialize(Map<String, Object?> json) => NewOrder(
        item: switch (json['item']) {
          final String item => item,
          null => throw const FormatException('item is required'),
          _ => throw const FormatException('item must be a string'),
        },
        quantity: switch (json['quantity']) {
          final int quantity => quantity,
          null => 1,
          _ => throw const FormatException('quantity must be an integer'),
        },
      );

  final String item;
  final int quantity;

  @override
  ValidationResult validate() {
    final errors = <ValidationError>[
      if (item.trim().isEmpty)
        const ValidationError(field: 'item', message: 'is required'),
      if (quantity < 1 || quantity > 10)
        const ValidationError(field: 'quantity', message: 'must be 1 to 10'),
    ];

    return errors.isEmpty ? const Valid() : Invalid(errors);
  }

  @override
  void validateOrThrow() {
    if (validate() case final Invalid invalid) {
      throw ValidationException(invalid.errors);
    }
  }
}

/// One placed order.
final class Order implements Serializable {
  const Order(this.id, this.item, this.quantity);

  final String id;
  final String item;
  final int quantity;

  @override
  Map<String, Object?> serialize() =>
      {'id': id, 'item': item, 'quantity': quantity};

  @override
  Map<String, Object?> toJson() => serialize();
}

/// Where orders live, attached with `withState`.
final class OrderStore {
  OrderStore([List<Order>? seed]) : orders = [...?seed];

  final List<Order> orders;

  Order? find(String id) => orders.where((order) => order.id == id).firstOrNull;
}

// --- the annotated handlers -------------------------------------------------

@GET('/', summary: 'List orders')
Future<List<Order>> listOrders(
  @Query('item') String? item,
  @State() OrderStore store,
) async {
  if (item == null) return store.orders;

  return store.orders.where((order) => order.item == item).toList();
}

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

@POST('/', status: 201)
Future<Order> placeOrder(
  @Extract(BearerAuth) AuthUser user,
  @State() OrderStore store,
  @Body() NewOrder input,
) async {
  final order = Order('${store.orders.length + 1}', input.item, input.quantity);
  store.orders.add(order);

  return order;
}

@DELETE('/{id}', status: 204)
Future<void> cancelOrder(
  @Extract(TodosWrite) AuthUser user,
  @Path() String id,
  @State() OrderStore store,
) async {
  store.orders.removeWhere((order) => order.id == id);
}
