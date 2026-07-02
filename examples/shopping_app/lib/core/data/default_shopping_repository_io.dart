import '../db/shopping_cache_database.dart';
import 'cached_shopping_repository.dart';
import 'shopping_repository.dart';

/// Creates default shopping repository.
ShoppingRepository createDefaultShoppingRepository() {
  return CachedShoppingRepository(
    remote: LiveShoppingRepository(),
    database: ShoppingCacheDatabase.open(':memory:'),
  );
}

/// Closes default shopping repository.
Future<void> closeDefaultShoppingRepository(
  ShoppingRepository repository,
) async {
  if (repository is CachedShoppingRepository) {
    await repository.close();
  }
}
