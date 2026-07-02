import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/core/data/shopping_repository.dart';
import 'package:shopping_app/features/products/models/products_state.dart';
import 'package:shopping_app/features/products/view_models/products_view_model.dart';

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

class LifecycleProductsViewModel extends ProductsViewModel {
  LifecycleProductsViewModel(super.args, this.label);

  final String label;
  var disposeCalls = 0;

  @override
  Future<void> onInit() async {
    emit(state.copyWith(searchQuery: label));
  }

  void emitForTest(ProductsState state) {
    emit(state);
  }

  @override
  void dispose() {
    disposeCalls += 1;
    super.dispose();
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
  testWidgets('selector resubscribes when value scope swaps view model', (
    tester,
  ) async {
    final first = TestProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
    );
    final second = TestProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
    );
    var selectorRebuilds = 0;

    Widget build(TestProductsViewModel viewModel) {
      return MaterialApp(
        home: ProductsViewModelScope.value(
          value: viewModel,
          child: ProductsViewModelSelector<String>(
            selector: (state) => state.searchQuery,
            builder: (context, searchQuery, child) {
              selectorRebuilds += 1;
              return Text(searchQuery.isEmpty ? 'empty' : searchQuery);
            },
          ),
        ),
      );
    }

    await tester.pumpWidget(build(first));

    expect(find.text('empty'), findsOneWidget);
    expect(selectorRebuilds, 1);

    first.emitForTest(first.state.copyWith(searchQuery: 'first'));
    await tester.pump();

    expect(find.text('first'), findsOneWidget);
    expect(selectorRebuilds, 2);

    await tester.pumpWidget(build(second));
    await tester.pump();

    expect(find.text('empty'), findsOneWidget);
    expect(selectorRebuilds, 3);

    first.emitForTest(first.state.copyWith(searchQuery: 'stale'));
    await tester.pump();

    expect(find.text('stale'), findsNothing);
    expect(selectorRebuilds, 3);

    second.emitForTest(second.state.copyWith(searchQuery: 'second'));
    await tester.pump();

    expect(find.text('second'), findsOneWidget);
    expect(selectorRebuilds, 4);
  });

  testWidgets('owned scope disposes replaced view models once', (
    tester,
  ) async {
    final created = <LifecycleProductsViewModel>[];

    Widget build(String identity) {
      return MaterialApp(
        home: TestIdentityScope(
          value: identity,
          child: ProductsViewModelScope(
            identity: TestIdentityScope.of,
            args: (_) => ProductsViewModelArgs(repository: MockRepository()),
            create: (context, args) {
              final viewModel = LifecycleProductsViewModel(
                args,
                TestIdentityScope.of(context),
              );
              created.add(viewModel);
              return viewModel;
            },
            child: ProductsViewModelSelector<String>(
              selector: (state) => state.searchQuery,
              builder: (context, searchQuery, child) {
                return Text(searchQuery);
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
    expect(created, hasLength(1));
    final first = created.single;
    expect(first.disposeCalls, 0);

    await tester.pumpWidget(build('two'));
    await tester.pump();
    await tester.pump();

    expect(find.text('two'), findsOneWidget);
    expect(find.text('one'), findsNothing);
    expect(created, hasLength(2));
    expect(first.disposeCalls, 1);
    final second = created.last;
    expect(second.disposeCalls, 0);

    first.emitForTest(first.state.copyWith(searchQuery: 'stale'));
    await tester.pump();

    expect(find.text('stale'), findsNothing);
    expect(find.text('two'), findsOneWidget);
    expect(first.disposeCalls, 1);

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();

    expect(first.disposeCalls, 1);
    expect(second.disposeCalls, 1);
  });
}
