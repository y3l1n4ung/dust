import 'package:dust_dart/derive.dart';
import '../../../core/models/store_cart.dart';

part 'demo_cart_state.g.dart';

/// Demo cart status values for the shopping app example.
enum DemoCartStatus {
  /// Initial demo cart status.
  initial,

  /// Loading demo cart status.
  loading,

  /// Success demo cart status.
  success,

  /// Error demo cart status.
  error,
}

/// Demo cart state for the shopping app example.
@Derive([ToString(), CopyWith(), Eq()])
class DemoCartState with _$DemoCartState {
  /// Creates a [DemoCartState].
  const DemoCartState({
    this.status = DemoCartStatus.initial,
    this.carts = const [],
    this.errorMessage,
  });

  /// Status.
  final DemoCartStatus status;

  /// Carts.
  final List<StoreCart> carts;

  /// Error message.
  final String? errorMessage;
}
