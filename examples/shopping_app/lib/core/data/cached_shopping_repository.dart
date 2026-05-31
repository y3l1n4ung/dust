import '../../features/auth/models/user.dart';
import '../../features/checkout/models/checkout_quote.dart';
import '../../features/orders/models/order_tracking.dart';
import '../../features/product_detail/models/product_review.dart';
import '../../features/products/models/product.dart';
import '../../features/support/models/chat_socket.dart';
import '../db/shopping_cache_database.dart';
import '../logging/logger.dart';
import '../models/store_cart.dart';
import 'shopping_repository.dart';

final class CachedShoppingRepository implements ShoppingRepository {
  CachedShoppingRepository({
    required ShoppingRepository remote,
    required ShoppingCacheDatabase database,
  }) : _remote = remote,
       _database = database;

  final ShoppingRepository _remote;
  final ShoppingCacheDatabase _database;

  Future<void> close() async {
    await _database.pool.close();
  }

  @override
  Future<List<Product>> getProducts() async {
    try {
      final products = await _remote.getProducts();
      await _database.pool.replaceProductCache(products);
      return products;
    } catch (error) {
      logger.warning('DB', 'Using cached products after API failure: $error');
      final rows = await _database.pool.listCachedProducts();
      if (rows.isEmpty) rethrow;
      return rows.map((row) => row.toProduct()).toList(growable: false);
    }
  }

  @override
  Future<Product> getProduct(int id) async {
    try {
      final product = await _remote.getProduct(id);
      await _database.pool.saveProduct(product);
      return product;
    } catch (error) {
      logger.warning('DB', 'Using cached product #$id after API failure: $error');
      final row = await _database.pool.findCachedProduct(id);
      if (row == null) rethrow;
      return row.toProduct();
    }
  }

  @override
  Future<List<Product>> getProductsPage({int? limit, String? sort}) {
    return _remote.getProductsPage(limit: limit, sort: sort);
  }

  @override
  Future<List<Product>> getProductsByCategory(
    String category, {
    int? limit,
    String? sort,
  }) {
    return _remote.getProductsByCategory(category, limit: limit, sort: sort);
  }

  @override
  Future<List<String>> getCategories() => _remote.getCategories();

  @override
  Future<List<StoreCart>> getCarts({int? limit, String? sort}) {
    return _remote.getCarts(limit: limit, sort: sort);
  }

  @override
  Future<StoreCart> getCart(int id) => _remote.getCart(id);

  @override
  Future<List<StoreCart>> getUserCarts(int userId) => _remote.getUserCarts(userId);

  @override
  Future<String> login(String username, String password) {
    return _remote.login(username, password);
  }

  @override
  Future<User> getUser(int id) => _remote.getUser(id);

  @override
  Future<int> registerUser({
    required String email,
    required String username,
    required String password,
    required String firstName,
    required String lastName,
    required String phone,
  }) {
    return _remote.registerUser(
      email: email,
      username: username,
      password: password,
      firstName: firstName,
      lastName: lastName,
      phone: phone,
    );
  }

  @override
  Future<List<ProductReview>> getProductReviews(int productId) {
    return _remote.getProductReviews(productId);
  }

  @override
  Future<List<Product>> getRecommendations(int productId) {
    return _remote.getRecommendations(productId);
  }

  @override
  Future<CheckoutQuote> quoteCheckout(CheckoutQuoteRequest request) {
    return _remote.quoteCheckout(request);
  }

  @override
  Future<List<TrackingEvent>> getOrderTracking(String orderId) {
    return _remote.getOrderTracking(orderId);
  }

  @override
  ShoppingChatSocket openChatSocket() => _remote.openChatSocket();
}
