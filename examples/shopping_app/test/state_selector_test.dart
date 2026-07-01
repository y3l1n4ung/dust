import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/features/products/view_models/products_view_model.dart';
import 'package:shopping_app/features/products/models/products_state.dart';
import 'package:shopping_app/core/data/shopping_repository.dart';

class MockRepository implements ShoppingRepository {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class TestProductsViewModel extends ProductsViewModel {
  TestProductsViewModel(super.args);

  void emitForTest(ProductsState state) {
    emit(state);
  }
}

void main() {
  testWidgets('watch value rebuilds with typed state', (
    tester,
  ) async {
    var rebuilds = 0;
    final viewModel = TestProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: ProductsViewModelScope.value(
          value: viewModel,
          child: Column(
            children: [
              Builder(
                builder: (context) {
                  final state = context.watchProductsViewModel().value;
                  rebuilds++;
                  return Text('Status: ${state.status}');
                },
              ),
            ],
          ),
        ),
      ),
    );

    expect(find.text('Status: ProductsStatus.initial'), findsOneWidget);
    expect(rebuilds, 1);

    viewModel.emitForTest(viewModel.state.copyWith(searchQuery: 'backpack'));
    await tester.pump();

    expect(rebuilds, 2);

    viewModel.emitForTest(
      viewModel.state.copyWith(status: ProductsStatus.loading),
    );
    await tester.pump();

    expect(rebuilds, 3);
    expect(find.text('Status: ProductsStatus.loading'), findsOneWidget);
  });
}
