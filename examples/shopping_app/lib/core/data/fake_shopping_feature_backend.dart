import 'dart:async';

import '../../features/checkout/models/checkout_quote.dart';
import '../../features/orders/models/order_tracking.dart';
import '../../features/product_detail/models/product_review.dart';
import '../../features/support/models/chat_message.dart';
import '../../features/support/models/chat_socket.dart';

abstract interface class ShoppingFeatureBackend {
  Future<List<ProductReview>> getProductReviews(int productId);
  Future<CheckoutQuote> quoteCheckout(CheckoutQuoteRequest request);
  Future<List<TrackingEvent>> getOrderTracking(String orderId);
  ShoppingChatSocket openChatSocket();
}

final class FakeShoppingFeatureBackend implements ShoppingFeatureBackend {
  const FakeShoppingFeatureBackend();

  @override
  Future<List<ProductReview>> getProductReviews(int productId) async {
    await Future<void>.delayed(const Duration(milliseconds: 180));
    final now = DateTime.now();
    return [
      ProductReview(
        id: 'review-$productId-1',
        productId: productId,
        authorName: 'Avery Stone',
        rating: 4.8,
        comment: 'Fast shipping, clean packaging, and exactly as described.',
        createdAt: now.subtract(const Duration(days: 8)),
        verifiedPurchase: true,
      ),
      ProductReview(
        id: 'review-$productId-2',
        productId: productId,
        authorName: 'Maya Chen',
        rating: 4.5,
        comment: 'Good quality for the price. I would buy it again.',
        createdAt: now.subtract(const Duration(days: 21)),
        verifiedPurchase: true,
      ),
      ProductReview(
        id: 'review-$productId-3',
        productId: productId,
        authorName: 'Noah Rivera',
        rating: 4.2,
        comment: 'Solid product. The detail page helped me choose quickly.',
        createdAt: now.subtract(const Duration(days: 35)),
        verifiedPurchase: false,
      ),
    ];
  }

  @override
  Future<CheckoutQuote> quoteCheckout(CheckoutQuoteRequest request) async {
    await Future<void>.delayed(const Duration(milliseconds: 220));
    final code = request.couponCode?.trim().toUpperCase();
    final discount = switch (code) {
      'DUST10' => request.subtotal * 0.10,
      'SHIPFREE' => 0.0,
      null || '' => 0.0,
      _ => -1.0,
    };

    if (discount < 0) {
      return CheckoutQuote(
        subtotal: request.subtotal,
        discount: 0,
        shipping: 6.99,
        tax: request.subtotal * 0.07,
        total: request.subtotal + 6.99 + (request.subtotal * 0.07),
        estimatedDeliveryDays: 5,
        message: 'Coupon "$code" is not valid for this cart.',
      );
    }

    final shipping = code == 'SHIPFREE' ? 0.0 : 6.99;
    final taxable = request.subtotal - discount;
    final tax = taxable * 0.07;
    return CheckoutQuote(
      subtotal: request.subtotal,
      discount: discount,
      shipping: shipping,
      tax: tax,
      total: taxable + shipping + tax,
      estimatedDeliveryDays: shipping == 0 ? 6 : 4,
      appliedCoupon: code == null || code.isEmpty ? null : code,
      message: code == null || code.isEmpty
          ? 'Live FakeStore checkout with local quote preview.'
          : 'Coupon $code applied.',
    );
  }

  @override
  Future<List<TrackingEvent>> getOrderTracking(String orderId) async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    final now = DateTime.now();
    return [
      TrackingEvent(
        id: '$orderId-created',
        orderId: orderId,
        title: 'Order placed',
        description: 'We received your order and reserved the inventory.',
        location: 'Dust Store',
        occurredAt: now.subtract(const Duration(days: 2, hours: 4)),
        completed: true,
      ),
      TrackingEvent(
        id: '$orderId-packed',
        orderId: orderId,
        title: 'Packed',
        description: 'Your items passed quality check and were packed.',
        location: 'Yangon Fulfillment Hub',
        occurredAt: now.subtract(const Duration(days: 1, hours: 6)),
        completed: true,
      ),
      TrackingEvent(
        id: '$orderId-transit',
        orderId: orderId,
        title: 'In transit',
        description: 'The parcel is moving through the carrier network.',
        location: 'Mandalay Sorting Center',
        occurredAt: now.subtract(const Duration(hours: 8)),
        completed: true,
      ),
      TrackingEvent(
        id: '$orderId-delivery',
        orderId: orderId,
        title: 'Out for delivery',
        description: 'Expected delivery today before 6 PM.',
        location: 'Local courier route',
        occurredAt: now.add(const Duration(hours: 3)),
        completed: false,
      ),
    ];
  }

  @override
  ShoppingChatSocket openChatSocket() => _FakeShoppingChatSocket();

  static Future<ChatResponse> _buildChatResponse(ChatRequest request) async {
    await Future<void>.delayed(const Duration(milliseconds: 280));
    final text = request.message.toLowerCase();
    final reply = switch (text) {
      final value when value.contains('coupon') =>
        'Try DUST10 for 10% off or SHIPFREE for free delivery in the checkout quote demo.',
      final value when value.contains('order') || value.contains('track') =>
        'Open Orders, select an order, and the generated /orders/:orderId route shows fake tracking events.',
      final value when value.contains('review') =>
        'Product reviews are local fake responses so the UI is deterministic during tests.',
      final value when value.contains('api') =>
        'Products and carts use FakeStore through the generated Dio client; chat stays local for stability.',
      _ =>
        'I can help with products, coupons, order tracking, and the Dust codegen demo flow.',
    };

    return ChatResponse(
      message: ChatMessage(
        id: 'assistant-${DateTime.now().microsecondsSinceEpoch}',
        role: ChatRole.assistant,
        text: reply,
        createdAt: DateTime.now(),
      ),
      escalated: text.contains('human') || text.contains('agent'),
    );
  }
}

final class _FakeShoppingChatSocket implements ShoppingChatSocket {
  _FakeShoppingChatSocket() {
    _requestSub = _requests.stream.listen(_handleRequest);
  }

  final StreamController<ChatRequest> _requests =
      StreamController<ChatRequest>();
  final StreamController<ChatResponse> _responses =
      StreamController<ChatResponse>.broadcast();
  late final StreamSubscription<ChatRequest> _requestSub;
  bool _closed = false;

  @override
  Stream<ChatResponse> get responses => _responses.stream;

  @override
  void send(ChatRequest request) {
    if (_closed) return;
    _requests.add(request);
  }

  Future<void> _handleRequest(ChatRequest request) async {
    final response = await FakeShoppingFeatureBackend._buildChatResponse(
      request,
    );
    if (!_closed) {
      _responses.add(response);
    }
  }

  @override
  Future<void> close() async {
    if (_closed) return;
    _closed = true;
    await _requestSub.cancel();
    await _requests.close();
    await _responses.close();
  }
}
