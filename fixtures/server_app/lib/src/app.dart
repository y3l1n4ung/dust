import 'package:dust_server/server.dart';

import 'features/accounts/accounts_handler.dart';
import 'features/accounts/accounts_repo.dart';
import 'features/exports/exports_handler.dart';
import 'features/inventory/inventory_handler.dart';
import 'features/inventory/inventory_repo.dart';
import 'features/orders/orders_handler.dart';
import 'features/orders/orders_repo.dart';
import 'shared/db/database.dart';

/// Mounts every feature's generated router and attaches the state they ask for.
///
/// One module per feature, each emitted from the annotations in that feature's
/// `*_handler.dart` and named after the file it came from. This is the only
/// file that knows the shape of the whole application, and it knows nothing
/// about what any handler does.
///
/// Repositories are attached by type, which is how `@State()` finds them. They
/// wrap [AppDatabase.connection] rather than owning it, so building one is
/// free and the application closes the database exactly once.
Router buildApp(AppDatabase database) {
  final connection = database.connection;

  return Router()
    ..nest('/auth', $accountsHandlerRoutes())
    ..nest('/orders', $ordersHandlerRoutes())
    ..nest('/inventory', $inventoryHandlerRoutes())
    ..nest('/exports', $exportsHandlerRoutes())
    ..withState(AccountsRepo(connection))
    ..withState(OrdersRepo(connection))
    ..withState(InventoryRepo(connection))
    ..withState(database);
}
