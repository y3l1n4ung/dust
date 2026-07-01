import 'package:dust_flutter/state.dart';
import 'package:dust_dart/serde.dart';

import '../data/shopping_repository.dart';
import '../services/storage_service.dart';
import '../../features/products/models/product.dart';

part 'app_view_model.g.dart';

enum AppBackendMode { liveFakeStore }

@Derive([ToString(), Eq(), CopyWith()])
class AppState with _$AppState {
  const AppState({this.backendMode = AppBackendMode.liveFakeStore});

  final AppBackendMode backendMode;
}

final class AppViewModelArgs extends ViewModelArgs {
  const AppViewModelArgs({
    required this.repository,
    required this.storage,
    super.observer,
  });

  final ShoppingRepository repository;
  final StorageService storage;
}

@ViewModel(state: AppState, args: AppViewModelArgs)
class AppViewModel extends $AppViewModel {
  AppViewModel(super.args);
}

enum BnbTab { home, products, cart, orders, profile }

@Derive([ToString(), Eq(), CopyWith()])
class BnbState with _$BnbState {
  const BnbState({this.currentTab = BnbTab.home});

  final BnbTab currentTab;

  int get currentIndex => currentTab.index;
}

final class BnbViewModelArgs extends ViewModelArgs {
  const BnbViewModelArgs({super.observer});
}

@ViewModel(state: BnbState, args: BnbViewModelArgs)
class BnbViewModel extends $BnbViewModel {
  BnbViewModel(super.args);

  void select(BnbTab tab) {
    if (tab == state.currentTab) return;
    emit(state.copyWith(currentTab: tab));
  }

  void selectIndex(int index) {
    RangeError.checkValidIndex(index, BnbTab.values);
    select(BnbTab.values[index]);
  }
}

@Derive([ToString(), Eq()])
class HomePageData with _$HomePageData {
  const HomePageData({
    required this.featuredProducts,
    required this.categories,
  });

  final List<Product> featuredProducts;
  final List<String> categories;
}

final class HomeViewModelArgs extends ViewModelArgs {
  const HomeViewModelArgs({required this.repository, super.observer});

  final ShoppingRepository repository;
}

@ViewModel(
  state: HomePageData,
  args: HomeViewModelArgs,
  mode: ViewModelMode.async,
)
class HomeViewModel extends $HomeViewModel {
  HomeViewModel(super.args);

  @override
  Future<HomePageData> loadData() async {
    final products = await args.repository.getProductsPage(limit: 6);
    final categories = await args.repository.getCategories();
    return HomePageData(
      featuredProducts: products,
      categories: categories,
    );
  }
}
