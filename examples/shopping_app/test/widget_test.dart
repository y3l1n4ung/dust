import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:shopping_app/core/data/shopping_repository.dart';
import 'package:shopping_app/core/services/storage_service.dart';
import 'package:shopping_app/features/auth/models/user.dart';
import 'package:shopping_app/features/products/models/product.dart';
import 'package:shopping_app/main.dart';

void main() {
  testWidgets('Shopping app loads with Dust scopes and router', (tester) async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();

    await tester.pumpWidget(
      ShoppingApp(
        storage: StorageService(prefs),
        repository: const _FakeShoppingRepository(),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Shop'), findsOneWidget);
  });
}

final class _FakeShoppingRepository implements ShoppingRepository {
  const _FakeShoppingRepository();

  @override
  Future<List<Product>> getProducts() async => const [
    Product(
      id: 1,
      title: 'Dust Backpack',
      price: 42,
      description: 'A generated shopping example product.',
      category: 'bags',
      image: 'https://example.com/backpack.png',
      rating: Rating(rate: 4.8, count: 12),
    ),
  ];

  @override
  Future<Product> getProduct(int id) async => (await getProducts()).first;

  @override
  Future<List<String>> getCategories() async => const ['bags'];

  @override
  Future<String> login(String username, String password) async => 'token';

  @override
  Future<User> getUser(int id) async => const User(
    id: 1,
    email: 'dust@example.com',
    username: 'dust',
    name: Name(firstname: 'Dust', lastname: 'User'),
    phone: '555-0100',
  );

  @override
  Future<int> registerUser({
    required String email,
    required String username,
    required String password,
    required String firstName,
    required String lastName,
    required String phone,
  }) async => 1;
}
