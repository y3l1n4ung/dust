import '../../../core/models/store_cart.dart';

enum DemoCartStatus { initial, loading, success, error }

class DemoCartState {
  const DemoCartState({
    this.status = DemoCartStatus.initial,
    this.carts = const [],
    this.errorMessage,
  });

  final DemoCartStatus status;
  final List<StoreCart> carts;
  final String? errorMessage;

  DemoCartState copyWith({
    DemoCartStatus? status,
    List<StoreCart>? carts,
    String? errorMessage,
  }) {
    return DemoCartState(
      status: status ?? this.status,
      carts: carts ?? this.carts,
      errorMessage: errorMessage,
    );
  }
}
