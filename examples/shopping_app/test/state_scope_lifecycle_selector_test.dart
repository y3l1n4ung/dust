import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/features/products/view_models/products_view_model.dart';
import 'package:shopping_app/features/products/models/products_state.dart';
import 'state_selector_support.dart';

/// Scope lifecycle: init, identity changes and listener resubscription.
void main() {
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

  testWidgets('.value scope does not retry failed init on parent rebuild', (
    tester,
  ) async {
    final viewModel = FailingInitProductsViewModel(
      ProductsViewModelArgs(repository: MockRepository()),
    );

    Widget build() {
      return MaterialApp(
        home: ProductsViewModelScope.value(
          value: viewModel,
          child: const SizedBox(),
        ),
      );
    }

    await tester.pumpWidget(build());
    await tester.pump();

    expect(viewModel.initCalls, 1);

    viewModel.initCompleters.single.completeError(StateError('init failed'));
    await tester.pump();

    expect(tester.takeException(), isA<StateError>());

    await tester.pumpWidget(build());
    await tester.pump();

    expect(viewModel.initCalls, 1);
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
