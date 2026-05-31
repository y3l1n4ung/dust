import 'package:derive_annotation/derive_annotation.dart';
import '../../../core/models/store_cart.dart';

part 'demo_cart_state.g.dart';

enum DemoCartStatus { initial, loading, success, error }

@Derive([ToString(), CopyWith(), Eq()])
class DemoCartState with _$DemoCartState {
  const DemoCartState({
    this.status = DemoCartStatus.initial,
    this.carts = const [],
    this.errorMessage,
  });

  final DemoCartStatus status;
  final List<StoreCart> carts;
  final String? errorMessage;
}
