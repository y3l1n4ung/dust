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

  void emitEffectForTest(Object effect) {
    emitEffect(effect);
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

    await tester.pumpWidget(build('two'));
    await tester.pump();
    await tester.pump();

    expect(find.text('two'), findsOneWidget);
    expect(find.text('one'), findsNothing);
  });

  testWidgets('listener resubscribes when value scope swaps view model', (
    tester,
  ) async {
    final first = TestProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
    );
    final second = TestProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
    );
    final effects = <Object>[];

    Widget build(TestProductsViewModel viewModel) {
      return MaterialApp(
        home: ProductsViewModelScope.value(
          value: viewModel,
          child: ProductsViewModelListener(
            listener: (context, effect) => effects.add(effect),
            child: const SizedBox.shrink(),
          ),
        ),
      );
    }

    await tester.pumpWidget(build(first));
    await tester.pump();

    first.emitEffectForTest('first');
    await tester.pump();

    expect(effects, <Object>['first']);

    await tester.pumpWidget(build(second));
    await tester.pump();

    first.emitEffectForTest('stale');
    second.emitEffectForTest('second');
    await tester.pump();

    expect(effects, <Object>['first', 'second']);
  });

  testWidgets('listener does not rebuild child for state changes or effects', (
    tester,
  ) async {
    var childBuilds = 0;
    final effects = <Object>[];
    final viewModel = TestProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
    );

    await tester.pumpWidget(
      MaterialApp(
        home: ProductsViewModelScope.value(
          value: viewModel,
          child: ProductsViewModelListener(
            listener: (context, effect) => effects.add(effect),
            child: Builder(
              builder: (context) {
                childBuilds += 1;
                return const Text('listener child');
              },
            ),
          ),
        ),
      ),
    );

    expect(childBuilds, 1);

    viewModel.emitForTest(
      viewModel.state.copyWith(status: ProductsStatus.loading),
    );
    await tester.pump();

    expect(childBuilds, 1);
    expect(effects, isEmpty);

    viewModel.emitEffectForTest('toast');
    await tester.pump();

    expect(childBuilds, 1);
    expect(effects, <Object>['toast']);
  });
}
