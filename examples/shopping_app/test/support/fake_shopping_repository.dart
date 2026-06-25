import 'package:shopping_app/core/data/fake_shopping_feature_backend.dart';
import 'package:shopping_app/core/data/shopping_repository.dart';
import 'package:shopping_app/core/models/store_cart.dart';
import 'package:shopping_app/features/auth/models/user.dart';
import 'package:shopping_app/features/checkout/models/checkout_quote.dart';
import 'package:shopping_app/features/orders/models/order_tracking.dart';
import 'package:shopping_app/features/product_detail/models/product_review.dart';
import 'package:shopping_app/features/products/models/product.dart';
import 'package:shopping_app/features/support/models/chat_socket.dart';

final class FakeShoppingRepository implements ShoppingRepository {
  FakeShoppingRepository();

  int registerCalls = 0;
  String? lastRegisteredEmail;

  static const products = [
    Product(
      id: 1,
      title: 'Dust Backpack',
      price: 42,
      description: 'A generated shopping example product.',
      category: 'bags',
      image: 'https://example.com/backpack.png',
      rating: Rating(rate: 4.8, count: 12),
    ),
    Product(
      id: 2,
      title: 'Dust Hoodie',
      price: 64,
      description: 'A warm hoodie for code generation demos.',
      category: 'clothing',
      image: 'https://example.com/hoodie.png',
      rating: Rating(rate: 4.6, count: 18),
    ),
  ];

  @override
  Future<List<Product>> getProducts() async => products;

  @override
  Future<List<Product>> getProductsPage({int? limit, String? sort}) async =>
      products.take(limit ?? products.length).toList();

  @override
  Future<Product> getProduct(int id) async => products.first;

  @override
  Future<List<Product>> getProductsByCategory(
    String category, {
    int? limit,
    String? sort,
  }) async =>
      products
          .where((product) => product.category == category)
          .take(limit ?? products.length)
          .toList();

  @override
  Future<List<String>> getCategories() async => const ['bags', 'clothing'];

  @override
  Future<List<StoreCart>> getCarts({int? limit, String? sort}) async =>
      (await getUserCarts(1)).take(limit ?? 99).toList();

  @override
  Future<StoreCart> getCart(int id) async => (await getUserCarts(1)).first;

  @override
  Future<List<StoreCart>> getUserCarts(int userId) async => [
        StoreCart(
          id: 1,
          userId: userId,
          date: DateTime(2026, 1, 1),
          products: const [StoreCartProduct(productId: 1, quantity: 2)],
        ),
      ];

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
  }) async {
    registerCalls += 1;
    lastRegisteredEmail = email;
    return 1;
  }

  @override
  Future<List<ProductReview>> getProductReviews(int productId) async => [
        ProductReview(
          id: 'review-1',
          productId: productId,
          authorName: 'Dust Tester',
          rating: 4.9,
          comment: 'Generated reviews work.',
          createdAt: DateTime(2026, 1, 2),
          verifiedPurchase: true,
        ),
      ];

  @override
  Future<List<Product>> getRecommendations(int productId) async => products
      .where((product) => product.id != productId)
      .toList(growable: false);

  @override
  Future<CheckoutQuote> quoteCheckout(CheckoutQuoteRequest request) async {
    final discount =
        request.couponCode == 'DUST10' ? request.subtotal * 0.1 : 0.0;
    return CheckoutQuote(
      subtotal: request.subtotal,
      discount: discount,
      shipping: 5,
      tax: 2,
      total: request.subtotal - discount + 7,
      estimatedDeliveryDays: 4,
      appliedCoupon: request.couponCode,
    );
  }

  @override
  Future<List<TrackingEvent>> getOrderTracking(String orderId) async => [
        TrackingEvent(
          id: 'tracking-1',
          orderId: orderId,
          title: 'Packed',
          description: 'Packed for delivery.',
          location: 'Dust warehouse',
          occurredAt: DateTime(2026, 1, 3),
          completed: true,
        ),
      ];

  @override
  ShoppingChatSocket openChatSocket() =>
      const FakeShoppingFeatureBackend().openChatSocket();
}
