import 'package:dust_flutter/state.dart';

import '../../../core/data/shopping_repository.dart';
import '../models/product_detail_state.dart';

part 'product_detail_view_model.g.dart';

/// Product detail view model args model for the shopping app example.
final class ProductDetailViewModelArgs extends ViewModelArgs {
  /// Creates a [ProductDetailViewModelArgs].
  const ProductDetailViewModelArgs({required this.repository, super.observer});

  /// Repository.
  final ShoppingRepository repository;
}

/// Product detail view model for the shopping app example.
@ViewModel(state: ProductDetailState, args: ProductDetailViewModelArgs)
class ProductDetailViewModel extends $ProductDetailViewModel {
  /// Creates a [ProductDetailViewModel].
  ProductDetailViewModel(super.args);

  /// Loads.
  Future<void> load(int productId) async {
    if (state.productId == productId &&
        state.status == ProductDetailStatus.success) {
      return;
    }

    emit(
      state.copyWith(
        productId: productId,
        status: ProductDetailStatus.loading,
        errorMessage: null,
      ),
    );

    try {
      final reviews = await args.repository.getProductReviews(productId);
      final recommendations = await args.repository.getRecommendations(
        productId,
      );
      emit(
        state.copyWith(
          productId: productId,
          status: ProductDetailStatus.success,
          reviews: reviews,
          recommendations: recommendations,
          errorMessage: null,
        ),
      );
    } catch (error) {
      emit(
        state.copyWith(
          productId: productId,
          status: ProductDetailStatus.error,
          errorMessage: error.toString(),
        ),
      );
    }
  }
}
