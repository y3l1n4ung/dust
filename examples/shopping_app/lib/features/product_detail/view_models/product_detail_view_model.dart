import 'package:dust_flutter/state.dart';

import '../../../core/data/shopping_repository.dart';
import '../models/product_detail_state.dart';

part 'product_detail_view_model.g.dart';

final class ProductDetailViewModelArgs extends ViewModelArgs {
  const ProductDetailViewModelArgs({required this.repository, super.observer});

  final ShoppingRepository repository;
}

@ViewModel(state: ProductDetailState, args: ProductDetailViewModelArgs)
class ProductDetailViewModel extends $ProductDetailViewModel {
  ProductDetailViewModel(super.args);

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
      final recommendations = await args.repository.getRecommendations(productId);
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
