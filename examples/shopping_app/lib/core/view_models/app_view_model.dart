import 'package:dust_flutter/state.dart';
import 'package:dust_dart/serde.dart';

import '../data/shopping_repository.dart';
import '../services/storage_service.dart';
import '../../features/products/models/product.dart';

part 'app_view_model.g.dart';

/// App backend mode values for the shopping app example.
enum AppBackendMode {
  /// Live fake store app backend mode.
  liveFakeStore,
}

/// App state for the shopping app example.
@Derive([ToString(), Eq(), CopyWith()])
class AppState with _$AppState {
  /// Creates an [AppState].
  const AppState({this.backendMode = AppBackendMode.liveFakeStore});

  /// Backend mode.
  final AppBackendMode backendMode;
}

/// App view model args model for the shopping app example.
final class AppViewModelArgs extends ViewModelArgs {
  /// Creates an [AppViewModelArgs].
  const AppViewModelArgs({
    required this.repository,
    required this.storage,
    super.observer,
  });

  /// Repository.
  final ShoppingRepository repository;

  /// Storage.
  final StorageService storage;
}

/// App view model for the shopping app example.
@ViewModel(state: AppState, args: AppViewModelArgs)
class AppViewModel extends $AppViewModel {
  /// Creates an [AppViewModel].
  AppViewModel(super.args);
}

/// Bnb tab values for the shopping app example.
enum BnbTab {
  /// Home bnb tab.
  home,

  /// Products bnb tab.
  products,

  /// Cart bnb tab.
  cart,

  /// Orders bnb tab.
  orders,

  /// Profile bnb tab.
  profile,
}

/// Bnb state for the shopping app example.
@Derive([ToString(), Eq(), CopyWith()])
class BnbState with _$BnbState {
  /// Creates a [BnbState].
  const BnbState({this.currentTab = BnbTab.home});

  /// Current tab.
  final BnbTab currentTab;

  /// Current index.
  int get currentIndex => currentTab.index;
}

/// Bnb view model args model for the shopping app example.
final class BnbViewModelArgs extends ViewModelArgs {
  /// Creates a [BnbViewModelArgs].
  const BnbViewModelArgs({super.observer});
}

/// Bnb view model for the shopping app example.
@ViewModel(state: BnbState, args: BnbViewModelArgs)
class BnbViewModel extends $BnbViewModel {
  /// Creates a [BnbViewModel].
  BnbViewModel(super.args);

  /// Selects.
  void select(BnbTab tab) {
    if (tab == state.currentTab) return;

    emit(state.copyWith(currentTab: tab));
  }

  /// Selects index.
  void selectIndex(int index) {
    RangeError.checkValidIndex(index, BnbTab.values);

    select(BnbTab.values[index]);
  }
}

/// Home page data model for the shopping app example.
@Derive([ToString(), Eq()])
class HomePageData with _$HomePageData {
  /// Creates a [HomePageData].
  const HomePageData({
    required this.featuredProducts,
    required this.categories,
  });

  /// Featured products.
  final List<Product> featuredProducts;

  /// Categories.
  final List<String> categories;
}

/// Home view model args model for the shopping app example.
final class HomeViewModelArgs extends ViewModelArgs {
  /// Creates a [HomeViewModelArgs].
  const HomeViewModelArgs({required this.repository, super.observer});

  /// Repository.
  final ShoppingRepository repository;
}

/// Home view model for the shopping app example.
@ViewModel(
  state: HomePageData,
  args: HomeViewModelArgs,
  mode: ViewModelMode.async,
)
class HomeViewModel extends $HomeViewModel {
  /// Creates a [HomeViewModel].
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
