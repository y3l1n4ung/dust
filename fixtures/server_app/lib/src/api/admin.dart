import 'package:dust_server/server.dart';

import '../auth/caller.dart';
import '../auth/require_scope.dart';
import '../models/order.dart';
import '../repo/order_store.dart';

part 'admin.g.dart';

// One extractor applied to every route in the module, rather than named on each
// handler. The plugin emits `routeLayer(fromExtractor(...))` for it, and the
// handlers read the value back with `Extension<T>` — axum's `from_extractor`
// and `Extension`, which is why there is no new vocabulary here.
//
// A route added to this file later is covered without anyone remembering to
// cover it.

/// `GET /admin/orders` — every order, for staff.
@GET('/orders', summary: 'Every order')
Future<List<Order>> allOrders(
  @Extract(RequireScope) Caller caller,
  @State() OrderStore store,
) async {
  return store.orders;
}

/// `GET /admin/whoami` — proof the guard ran.
@GET('/whoami')
Future<Map<String, Object?>> whoAmI(
  @Extract(RequireScope) Caller caller,
) async {
  return {'id': caller.id, 'scopes': caller.scopes};
}

/// `DELETE /admin/orders` — clears the lot.
@DELETE('/orders', status: 204)
Future<void> clearOrders(
  @Extract(RequireScope) Caller caller,
  @State() OrderStore store,
) async {
  store.orders.clear();
}
