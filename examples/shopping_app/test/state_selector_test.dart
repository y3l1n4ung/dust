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

  @override
  Future<void> onInit() async {}

  void emitForTest(ProductsState state) {
    emit(state);
  }
}

class InitProductsViewModel extends ProductsViewModel {
  InitProductsViewModel(super.args, this.label);

  final String label;

  @override
  Future<void> onInit() async {
    emit(state.copyWith(searchQuery: '${state.searchQuery}$label'));
  }
}

class TestIdentityScope extends InheritedWidget {
  const TestIdentityScope({
    required this.value,
    required super.child,
    super.key,
  });

  final String value;

  static String of(BuildContext context) {
    final scope =
        context.dependOnInheritedWidgetOfExactType<TestIdentityScope>();
    if (scope == null) throw StateError('No TestIdentityScope found.');
    return scope.value;
  }

  @override
  bool updateShouldNotify(TestIdentityScope oldWidget) {
    return value != oldWidget.value;
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

  testWidgets('.value scope runs init once for external view model', (
    tester,
  ) async {
    final viewModel = InitProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
      'external',
    );

    Widget build() {
      return MaterialApp(
        home: ProductsViewModelScope.value(
          value: viewModel,
          child: Builder(
            builder: (context) {
              final state = context.watchProductsViewModel().value;
              return Text(state.searchQuery);
            },
          ),
        ),
      );
    }

    await tester.pumpWidget(build());
    await tester.pump();
    await tester.pump();

    expect(find.text('external'), findsOneWidget);

    await tester.pumpWidget(build());
    await tester.pump();
    await tester.pump();

    expect(find.text('external'), findsOneWidget);
    expect(find.text('externalexternal'), findsNothing);
  });

  testWidgets('owned scope recreates when identity changes', (tester) async {
    Widget build(String identity) {
      return MaterialApp(
        home: TestIdentityScope(
          value: identity,
          child: ProductsViewModelScope(
            identity: TestIdentityScope.of,
            args: (_) => ProductsViewModelArgs(repository: MockRepository()),
            create: (context, args) {
              return InitProductsViewModel(args, TestIdentityScope.of(context));
            },
            child: Builder(
              builder: (context) {
                final state = context.watchProductsViewModel().value;
                return Text(state.searchQuery);
              },
            ),
          ),
        ),
      );
    }

    await tester.pumpWidget(build('one'));
    await tester.pump();
    await tester.pump();

    expect(find.text('one'), findsOneWidget);

    await tester.pumpWidget(build('two'));
    await tester.pump();
    await tester.pump();

    expect(find.text('two'), findsOneWidget);
    expect(find.text('one'), findsNothing);
  });
}
