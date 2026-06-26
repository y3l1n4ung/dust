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
  testWidgets('state aspects rebuild only when selected field changes', (
    tester,
  ) async {
    var selectRebuilds = 0;
    var fieldRebuilds = 0;
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
                  final status = context.watchProductsViewModel().select(
                        (state) => state.status,
                      );
                  selectRebuilds++;
                  return Text('Select status: $status');
                },
              ),
              Builder(
                builder: (context) {
                  final status = context.watchProductsViewModel().status;
                  fieldRebuilds++;
                  return Text('Field status: $status');
                },
              ),
            ],
          ),
        ),
      ),
    );

    expect(find.text('Select status: ProductsStatus.initial'), findsOneWidget);
    expect(find.text('Field status: ProductsStatus.initial'), findsOneWidget);
    expect(selectRebuilds, 1);
    expect(fieldRebuilds, 1);

    viewModel.emitForTest(viewModel.state.copyWith(searchQuery: 'backpack'));
    await tester.pump();

    expect(selectRebuilds, 1);
    expect(fieldRebuilds, 1);

    viewModel.emitForTest(
      viewModel.state.copyWith(status: ProductsStatus.loading),
    );
    await tester.pump();

    expect(selectRebuilds, 2);
    expect(fieldRebuilds, 2);
    expect(find.text('Select status: ProductsStatus.loading'), findsOneWidget);
    expect(find.text('Field status: ProductsStatus.loading'), findsOneWidget);
  });
}
