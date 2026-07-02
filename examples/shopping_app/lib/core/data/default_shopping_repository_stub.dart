import 'shopping_repository.dart';

/// Creates default shopping repository.
ShoppingRepository createDefaultShoppingRepository() {
  return LiveShoppingRepository();
}

/// Closes default shopping repository.
Future<void> closeDefaultShoppingRepository(
  ShoppingRepository repository,
) async {}
