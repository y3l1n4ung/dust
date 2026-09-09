import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/features/products/view_models/products_view_model.dart';
import 'package:shopping_app/features/products/models/products_state.dart';
import 'state_selector_support.dart';

void main() {
  testWidgets('selector rebuilds only when selected value changes', (
    tester,
  ) async {
    var fullRebuilds = 0;
    var selectorRebuilds = 0;
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
                  fullRebuilds++;
                  return Text('Full: ${state.status}');
                },
              ),
              ProductsViewModelSelector<ProductsStatus>(
                selector: (state) => state.status,
                builder: (context, status, child) {
                  selectorRebuilds++;
                  return Text('Selected: $status');
                },
              ),
            ],
          ),
        ),
      ),
    );

    expect(find.text('Full: ProductsStatus.initial'), findsOneWidget);
    expect(find.text('Selected: ProductsStatus.initial'), findsOneWidget);
    expect(fullRebuilds, 1);
    expect(selectorRebuilds, 1);

    viewModel.emitForTest(viewModel.state.copyWith(searchQuery: 'backpack'));
    await tester.pump();

    expect(fullRebuilds, 2);
    expect(selectorRebuilds, 1);

    viewModel.emitForTest(
      viewModel.state.copyWith(status: ProductsStatus.loading),
    );
    await tester.pump();

    expect(fullRebuilds, 3);
    expect(selectorRebuilds, 2);
    expect(find.text('Selected: ProductsStatus.loading'), findsOneWidget);
  });

  testWidgets('selector supports custom equality', (tester) async {
    var selectorRebuilds = 0;
    final viewModel = TestProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: ProductsViewModelScope.value(
          value: viewModel,
          child: ProductsViewModelSelector<String>(
            selector: (state) => state.searchQuery,
            equals: (previous, next) => previous.length == next.length,
            builder: (context, searchQuery, child) {
              selectorRebuilds++;
              return Text(searchQuery);
            },
          ),
        ),
      ),
    );

    expect(selectorRebuilds, 1);

    viewModel.emitForTest(viewModel.state.copyWith(searchQuery: 'aa'));
    await tester.pump();

    expect(selectorRebuilds, 2);
    expect(find.text('aa'), findsOneWidget);

    viewModel.emitForTest(viewModel.state.copyWith(searchQuery: 'bb'));
    await tester.pump();

    expect(selectorRebuilds, 2);
    expect(find.text('aa'), findsOneWidget);

    viewModel.emitForTest(viewModel.state.copyWith(searchQuery: 'ccc'));
    await tester.pump();

    expect(selectorRebuilds, 3);
    expect(find.text('ccc'), findsOneWidget);
  });

  testWidgets('selector ignores many unrelated state changes', (
    tester,
  ) async {
    var selectorRebuilds = 0;
    final viewModel = TestProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: ProductsViewModelScope.value(
          value: viewModel,
          child: ProductsViewModelSelector<ProductsStatus>(
            selector: (state) => state.status,
            builder: (context, status, child) {
              selectorRebuilds += 1;
              return Text(status.name);
            },
          ),
        ),
      ),
    );

    expect(selectorRebuilds, 1);

    for (var i = 0; i < 50; i += 1) {
      viewModel.emitForTest(viewModel.state.copyWith(searchQuery: 'q$i'));
      await tester.pump();
    }

    expect(selectorRebuilds, 1);
    expect(find.text(ProductsStatus.initial.name), findsOneWidget);
  });
}
