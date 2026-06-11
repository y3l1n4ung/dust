import '../db/shopping_cache_database.dart';
import 'cached_shopping_repository.dart';
import 'shopping_repository.dart';

ShoppingRepository createDefaultShoppingRepository() {
  return CachedShoppingRepository(
    remote: LiveShoppingRepository(),
    database: ShoppingCacheDatabase.open(':memory:'),
  );
}

Future<void> closeDefaultShoppingRepository(
  ShoppingRepository repository,
) async {
  if (repository is CachedShoppingRepository) {
    await repository.close();
  }
}
