import 'dart:async';

import 'package:dust_flutter/state.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/core/data/shopping_repository.dart';
import 'package:shopping_app/core/view_models/app_view_model.dart';
import 'package:shopping_app/features/products/models/product.dart';

class LifecycleBnbViewModel extends BnbViewModel {
  LifecycleBnbViewModel(super.args);

  var disposeCalls = 0;

  @override
  void dispose() {
    disposeCalls += 1;
    super.dispose();
  }
}

class LifecycleHomeViewModel extends HomeViewModel {
  LifecycleHomeViewModel(super.args);

  var disposeCalls = 0;

  @override
  void dispose() {
    disposeCalls += 1;
    super.dispose();
  }
}

class ControlledLifecycleRepository implements ShoppingRepository {
  final productPageLoad = Completer<List<Product>>();
  var productPageCalls = 0;

  @override
  Future<List<Product>> getProductsPage({int? limit, String? sort}) {
    productPageCalls += 1;
    return productPageLoad.future;
  }

  @override
  Future<List<String>> getCategories() async => const ['bags'];

  void completeProducts() {
    if (!productPageLoad.isCompleted) {
      productPageLoad.complete(const <Product>[]);
    }
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  testWidgets('real-world scope group disposes bnb and pending home once', (
    tester,
  ) async {
    final repository = ControlledLifecycleRepository();
    final bnbViewModels = <LifecycleBnbViewModel>[];
    final homeViewModels = <LifecycleHomeViewModel>[];

    await tester.pumpWidget(
      MaterialApp(
        home: ViewModelScopes(
          scopes: [
            (child) => BnbViewModelScope(
                  args: (_) => const BnbViewModelArgs(),
                  create: (_, args) {
                    final viewModel = LifecycleBnbViewModel(args);
                    bnbViewModels.add(viewModel);
                    return viewModel;
                  },
                  child: child,
                ),
            (child) => HomeViewModelScope(
                  args: (_) => HomeViewModelArgs(repository: repository),
                  create: (_, args) {
                    final viewModel = LifecycleHomeViewModel(args);
                    homeViewModels.add(viewModel);
                    return viewModel;
                  },
                  child: child,
                ),
          ],
          child: Column(
            children: [
              BnbViewModelSelector<BnbTab>(
                selector: (state) => state.currentTab,
                builder: (context, tab, child) => Text('tab:${tab.name}'),
              ),
              HomeViewModelBuilder(
                loading: (context) => const Text('home:loading'),
                data: (context, data) => Text(
                  'home:${data.featuredProducts.length}',
                ),
                error: (context, error, previousData) =>
                    const Text('home:error'),
              ),
            ],
          ),
        ),
      ),
    );

    await tester.pump();

    expect(find.text('tab:home'), findsOneWidget);
    expect(find.text('home:loading'), findsOneWidget);
    expect(repository.productPageCalls, 1);
    expect(bnbViewModels, hasLength(1));
    expect(homeViewModels, hasLength(1));

    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump();

    expect(bnbViewModels.single.disposeCalls, 1);
    expect(homeViewModels.single.disposeCalls, 1);

    repository.completeProducts();
    await tester.pump();

    expect(bnbViewModels.single.disposeCalls, 1);
    expect(homeViewModels.single.disposeCalls, 1);
  });
}
