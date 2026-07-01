import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/core/view_models/app_view_model.dart';

import 'support/fake_shopping_repository.dart';

void main() {
  test('bnb view model selects tabs by type and index', () {
    final viewModel = BnbViewModel(const BnbViewModelArgs())
      ..select(BnbTab.cart);

    expect(viewModel.state.currentTab, BnbTab.cart);
    expect(viewModel.state.currentIndex, BnbTab.cart.index);

    viewModel.selectIndex(BnbTab.orders.index);

    expect(viewModel.state.currentTab, BnbTab.orders);
  });

  test('home view model loads page data through args repository', () async {
    final viewModel = HomeViewModel(
      HomeViewModelArgs(repository: FakeShoppingRepository()),
    );

    await viewModel.load();

    expect(viewModel.state.hasData, isTrue);
    expect(viewModel.data?.featuredProducts, FakeShoppingRepository.products);
    expect(viewModel.data?.categories, const ['bags', 'clothing']);
  });
}
