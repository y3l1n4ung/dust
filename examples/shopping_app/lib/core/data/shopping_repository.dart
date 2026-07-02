import 'package:dio/dio.dart';

import '../../features/checkout/models/checkout_quote.dart';
import '../../features/orders/models/order_tracking.dart';
import '../../features/product_detail/models/product_review.dart';
import '../../features/auth/models/user.dart';
import '../../features/products/models/product.dart';
import '../../features/support/models/chat_socket.dart';
import '../api/shopping_api.dart';
import '../logging/logger.dart';
import '../models/store_cart.dart';
import 'fake_shopping_feature_backend.dart';

/// Shopping repository.
abstract interface class ShoppingRepository {
  /// Gets products.
  Future<List<Product>> getProducts();

  /// Gets products page.
  Future<List<Product>> getProductsPage({int? limit, String? sort});

  /// Gets product.
  Future<Product> getProduct(int id);

  /// Gets products by category.
  Future<List<Product>> getProductsByCategory(
    String category, {
    int? limit,
    String? sort,
  });

  /// Gets categories.
  Future<List<String>> getCategories();

  /// Gets carts.
  Future<List<StoreCart>> getCarts({int? limit, String? sort});

  /// Gets cart.
  Future<StoreCart> getCart(int id);

  /// Gets user carts.
  Future<List<StoreCart>> getUserCarts(int userId);

  /// Logs in.
  Future<String> login(String username, String password);

  /// Gets user.
  Future<User> getUser(int id);

  /// Registers user.
  Future<int> registerUser({
    required String email,
    required String username,
    required String password,
    required String firstName,
    required String lastName,
    required String phone,
  });

  /// Gets product reviews.
  Future<List<ProductReview>> getProductReviews(int productId);

  /// Gets recommendations.
  Future<List<Product>> getRecommendations(int productId);

  /// Quotes checkout.
  Future<CheckoutQuote> quoteCheckout(CheckoutQuoteRequest request);

  /// Gets order tracking.
  Future<List<TrackingEvent>> getOrderTracking(String orderId);

  /// Opens chat socket.
  ShoppingChatSocket openChatSocket();
}

/// Live shopping repository model for the shopping app example.
final class LiveShoppingRepository implements ShoppingRepository {
  /// Creates a [LiveShoppingRepository].
  LiveShoppingRepository({
    ShoppingApi? api,
    ShoppingFeatureBackend? featureBackend,
  })  : _api = api ?? ShoppingApi(Dio()),
        _featureBackend = featureBackend ?? const FakeShoppingFeatureBackend();

  final ShoppingApi _api;
  final ShoppingFeatureBackend _featureBackend;

  @override
  Future<List<Product>> getProducts() async {
    logger.apiRequest('GET', '/products');
    final products = await _api.getProducts();
    logger.info('API', 'Loaded ${products.length} products');
    return products;
  }

  @override
  Future<List<Product>> getProductsPage({int? limit, String? sort}) async {
    logger.apiRequest('GET', '/products', {'limit': limit, 'sort': sort});
    return _api.getProductsPage(limit: limit, sort: sort);
  }

  @override
  Future<Product> getProduct(int id) async {
    logger.apiRequest('GET', '/products/$id');
    return _api.getProduct(id);
  }

  @override
  Future<List<Product>> getProductsByCategory(
    String category, {
    int? limit,
    String? sort,
  }) {
    logger.apiRequest('GET', '/products/category/$category', {
      'limit': limit,
      'sort': sort,
    });
    return _api.getProductsByCategory(category, limit: limit, sort: sort);
  }

  @override
  Future<List<String>> getCategories() async {
    logger.apiRequest('GET', '/products/categories');
    return _api.getCategories();
  }

  @override
  Future<List<StoreCart>> getCarts({int? limit, String? sort}) {
    logger.apiRequest('GET', '/carts', {'limit': limit, 'sort': sort});
    return _api.getCarts(limit: limit, sort: sort);
  }

  @override
  Future<StoreCart> getCart(int id) {
    logger.apiRequest('GET', '/carts/$id');
    return _api.getCart(id);
  }

  @override
  Future<List<StoreCart>> getUserCarts(int userId) {
    logger.apiRequest('GET', '/carts/user/$userId');
    return _api.getUserCarts(userId);
  }

  @override
  Future<String> login(String username, String password) async {
    logger.apiRequest('POST', '/auth/login', {'username': username});
    final response = await _api.login(
      LoginRequest(username: username, password: password),
    );
    logger.info('API', 'Login successful for user: $username');
    return response.token;
  }

  @override
  Future<User> getUser(int id) {
    logger.apiRequest('GET', '/users/$id');
    return _api.getUser(id);
  }

  @override
  Future<int> registerUser({
    required String email,
    required String username,
    required String password,
    required String firstName,
    required String lastName,
    required String phone,
  }) async {
    logger.apiRequest('POST', '/users', {'username': username, 'email': email});
    final response = await _api.registerUser(
      RegisterUserRequest(
        email: email,
        username: username,
        password: password,
        name: RegisterName(firstname: firstName, lastname: lastName),
        phone: phone,
        address: const RegisterAddress(
          city: '',
          street: '',
          number: 0,
          zipcode: '',
          geolocation: RegisterGeolocation(lat: '0', long: '0'),
        ),
      ),
    );
    logger.info('API', 'Registration successful for user: $username');
    return response.id;
  }

  @override
  Future<List<ProductReview>> getProductReviews(int productId) {
    logger.apiRequest('FAKE', '/products/$productId/reviews');
    return _featureBackend.getProductReviews(productId);
  }

  @override
  Future<List<Product>> getRecommendations(int productId) async {
    final products = await getProducts();
    final current = products.where((product) => product.id == productId);
    final category = current.isEmpty ? null : current.first.category;
    final recommendations = products
        .where(
          (product) =>
              product.id != productId &&
              (category == null || product.category == category),
        )
        .take(4)
        .toList();
    return recommendations.isEmpty
        ? products.where((product) => product.id != productId).take(4).toList()
        : recommendations;
  }

  @override
  Future<CheckoutQuote> quoteCheckout(CheckoutQuoteRequest request) {
    logger.apiRequest('FAKE', '/checkout/quote', {
      'subtotal': request.subtotal,
      'couponCode': request.couponCode,
    });
    return _featureBackend.quoteCheckout(request);
  }

  @override
  Future<List<TrackingEvent>> getOrderTracking(String orderId) {
    logger.apiRequest('FAKE', '/orders/$orderId/tracking');
    return _featureBackend.getOrderTracking(orderId);
  }

  @override
  ShoppingChatSocket openChatSocket() {
    logger.info('FAKE', 'Opening local support chat socket stream');
    return _featureBackend.openChatSocket();
  }
}
