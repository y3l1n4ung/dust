import 'shopping_repository.dart';

ShoppingRepository createDefaultShoppingRepository() {
  return LiveShoppingRepository();
}

Future<void> closeDefaultShoppingRepository(ShoppingRepository repository) async {}
