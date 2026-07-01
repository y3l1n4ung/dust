import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/core/view_models/app_view_model.dart';

import 'support/fake_shopping_repository.dart';

final class ControlledHomeViewModel extends HomeViewModel {
  ControlledHomeViewModel()
      : super(HomeViewModelArgs(repository: FakeShoppingRepository()));

  final loads = <Completer<HomePageData>>[];

  @override
  Future<HomePageData> loadData() {
    final completer = Completer<HomePageData>();
    loads.add(completer);
    return completer.future;
  }
}

const homeData = HomePageData(
  featuredProducts: FakeShoppingRepository.products,
  categories: ['bags', 'clothing'],
);

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

  testWidgets('home builder renders async lifecycle with previous data', (
    tester,
  ) async {
    final viewModel = ControlledHomeViewModel();

    await tester.pumpWidget(
      MaterialApp(
        home: HomeViewModelScope.value(
          value: viewModel,
          child: HomeViewModelBuilder(
            loading: (context) => const Text('loading'),
            data: (context, data) => Text(
              'data:${data.featuredProducts.length}:refreshing:${viewModel.state.isRefreshing}',
            ),
            error: (context, error, previousData) => Text(
              'error:${previousData?.featuredProducts.length}',
            ),
          ),
        ),
      ),
    );

    expect(find.text('loading'), findsOneWidget);

    await tester.pump();
    viewModel.loads.single.complete(homeData);
    await tester.pump();
    await tester.pump();

    expect(find.text('data:2:refreshing:false'), findsOneWidget);

    final refresh = viewModel.refresh();
    await tester.pump();

    expect(find.text('data:2:refreshing:true'), findsOneWidget);

    viewModel.loads.last.completeError(StateError('failed'));
    await refresh;
    await tester.pump();
    await tester.pump();

    expect(find.text('error:2'), findsOneWidget);
  });
}
