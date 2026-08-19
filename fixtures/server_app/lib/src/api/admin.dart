import 'package:dust_server/server.dart';

import '../auth/require_scope.dart';
import '../db/queries.dart';
import '../models/account.dart';
import '../models/order.dart';

part 'admin.g.dart';

// One extractor applied to every route in the module, rather than named on each
// handler. The plugin emits `routeLayer(fromExtractor(...))` for it, and the
// handlers read the value back with `Extension<T>` — axum's `from_extractor`
// and `Extension`, which is why there is no new vocabulary here.
//
// A route added to this file later is covered without anyone remembering to
// cover it.

/// `GET /admin/orders` — this account's orders, for staff tooling.
@GET('/orders', summary: 'Orders')
Future<Result<List<Order>, Rejection>> allOrders(
  @Extract(RequireScope) Account account,
  @State() AppQueries queries,
) async {
  return switch (await queries.ordersFor(account.id)) {
    Ok(:final value) => Ok(value),
    Err() => const Err(Rejection.internal()),
  };
}

/// `GET /admin/whoami` — proof the guard ran.
///
/// Answers with [AccountView], not [Account]: the stored type carries a
/// password hash and a salt, and a type that cannot be serialized cannot leak
/// them by accident.
@GET('/whoami')
Future<AccountView> whoAmI(
  @Extract(RequireScope) Account account,
) async {
  return AccountView.of(account);
}

/// `DELETE /admin/orders/{id}` — cancel one on a customer's behalf.
@DELETE('/orders/{id}', status: 204)
Future<Result<void, Rejection>> clearOrder(
  @Extract(RequireScope) Account account,
  @Path() int id,
  @State() AppQueries queries,
) async {
  return switch (await queries.deleteOrder(id, account.id)) {
    Ok() => const Ok(null),
    Err() => const Err(Rejection.internal()),
  };
}
