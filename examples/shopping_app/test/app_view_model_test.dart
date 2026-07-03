import 'dart:async';

import 'package:dust_flutter/state.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shopping_app/core/data/shopping_repository.dart';
import 'package:shopping_app/core/view_models/app_view_model.dart';
import 'package:shopping_app/features/products/models/product.dart';

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

final class ControlledHomeRepository implements ShoppingRepository {
  final productPageLoads = <Completer<List<Product>>>[];
  final categoryLoads = <Completer<List<String>>>[];

  int get productPageCalls => productPageLoads.length;
  int get categoryCalls => categoryLoads.length;

  @override
  Future<List<Product>> getProductsPage({int? limit, String? sort}) {
    final completer = Completer<List<Product>>();
    productPageLoads.add(completer);
    return completer.future;
  }

  @override
  Future<List<String>> getCategories() {
    final completer = Completer<List<String>>();
    categoryLoads.add(completer);
    return completer.future;
  }

  void completeProducts() {
    productPageLoads.single.complete(FakeShoppingRepository.products);
  }

  void completeCategories() {
    categoryLoads.single.complete(const ['bags', 'clothing']);
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
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

  testWidgets('bnb shell keeps async home state across tab switches', (
    tester,
  ) async {
    final repository = ControlledHomeRepository();
    final bnbViewModels = <BnbViewModel>[];
    final homeViewModels = <HomeViewModel>[];

    await tester.pumpWidget(
      MaterialApp(
        home: ViewModelScopes(
          scopes: [
            (child) => BnbViewModelScope(
                  args: (_) => const BnbViewModelArgs(),
                  create: (context, args) {
                    final viewModel = BnbViewModel(args);
                    bnbViewModels.add(viewModel);
                    return viewModel;
                  },
                  child: child,
                ),
            (child) => HomeViewModelScope(
                  args: (_) => HomeViewModelArgs(repository: repository),
                  create: (context, args) {
                    final viewModel = HomeViewModel(args);
                    homeViewModels.add(viewModel);
                    return viewModel;
                  },
                  child: child,
                ),
          ],
          child: const _BnbShellFixture(),
        ),
      ),
    );

    expect(find.text('home:loading'), findsOneWidget);
    expect(bnbViewModels.single.state.currentTab, BnbTab.home);

    await tester.pump();

    expect(repository.productPageCalls, 1);
    expect(repository.categoryCalls, 0);

    repository.completeProducts();
    await tester.pump();

    expect(repository.categoryCalls, 1);

    repository.completeCategories();
    await tester.pump();
    await tester.pump();

    expect(find.text('home:2:bags,clothing'), findsOneWidget);
    expect(homeViewModels, hasLength(1));

    await tester.tap(find.byKey(const ValueKey<String>('bnb-cart')));
    await tester.pump();

    expect(bnbViewModels.single.state.currentTab, BnbTab.cart);
    expect(find.text('cart tab'), findsOneWidget);
    expect(find.text('home:2:bags,clothing'), findsNothing);

    await tester.tap(find.byKey(const ValueKey<String>('bnb-home')));
    await tester.pump();

    expect(bnbViewModels.single.state.currentTab, BnbTab.home);
    expect(find.text('home:2:bags,clothing'), findsOneWidget);
    expect(repository.productPageCalls, 1);
    expect(repository.categoryCalls, 1);
    expect(homeViewModels, hasLength(1));
  });
}

final class _BnbShellFixture extends StatelessWidget {
  const _BnbShellFixture();

  @override
  Widget build(BuildContext context) {
    final tab = context.watchBnbViewModel().value.currentTab;
    return Scaffold(
      body: switch (tab) {
        BnbTab.home => const _HomeTabFixture(),
        BnbTab.products => const Center(child: Text('products tab')),
        BnbTab.cart => const Center(child: Text('cart tab')),
        BnbTab.orders => const Center(child: Text('orders tab')),
        BnbTab.profile => const Center(child: Text('profile tab')),
      },
      bottomNavigationBar: BottomNavigationBar(
        type: BottomNavigationBarType.fixed,
        currentIndex: tab.index,
        onTap: context.readBnbViewModel().selectIndex,
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.home, key: ValueKey<String>('bnb-home')),
            label: 'Home',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.storefront, key: ValueKey<String>('bnb-products')),
            label: 'Products',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.shopping_cart, key: ValueKey<String>('bnb-cart')),
            label: 'Cart',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.receipt_long, key: ValueKey<String>('bnb-orders')),
            label: 'Orders',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.person, key: ValueKey<String>('bnb-profile')),
            label: 'Profile',
          ),
        ],
      ),
    );
  }
}

final class _HomeTabFixture extends StatelessWidget {
  const _HomeTabFixture();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: HomeViewModelBuilder(
        loading: (context) => const Text('home:loading'),
        data: (context, data) => Text(
          'home:${data.featuredProducts.length}:${data.categories.join(',')}',
        ),
        error: (context, error, previousData) => Text(
          'home:error:${previousData?.featuredProducts.length ?? 0}',
        ),
      ),
    );
  }
}
