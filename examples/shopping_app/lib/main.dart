import 'package:flutter/material.dart';
import 'package:dust_state/dust_state.dart';
import 'package:flutter_web_plugins/url_strategy.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'core/data/shopping_repository.dart';
import 'core/services/storage_service.dart';
import 'core/view_models/app_view_model.dart';
import 'features/auth/view_models/auth_view_model.dart';
import 'features/cart/view_models/cart_view_model.dart';
import 'features/checkout/view_models/checkout_view_model.dart';
import 'features/demo_cart/view_models/demo_cart_api_view_model.dart';
import 'features/orders/view_models/orders_view_model.dart';
import 'features/orders/view_models/order_tracking_view_model.dart';
import 'features/product_detail/view_models/product_detail_view_model.dart';
import 'features/products/view_models/products_view_model.dart';
import 'features/support/view_models/shopping_chat_view_model.dart';
import 'features/wishlist/view_models/wishlist_view_model.dart';
import 'route.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  usePathUrlStrategy();
  final prefs = await SharedPreferences.getInstance();
  runApp(ShoppingApp(storage: StorageService(prefs)));
}

class ShoppingApp extends StatelessWidget {
  const ShoppingApp({required this.storage, this.repository, super.key});

  final StorageService storage;
  final ShoppingRepository? repository;

  @override
  Widget build(BuildContext context) {
    final observer = const LoggingStateObserver();
    return AppViewModelScope(
      args: (context) => AppViewModelArgs(
        repository: repository ?? LiveShoppingRepository(),
        storage: storage,
        observer: observer,
      ),
      create: (context, args) => AppViewModel(args),
      child: Builder(
        builder: (context) => AuthViewModelScope(
          args: (context) => AuthViewModelArgs(
            repository: context.readAppViewModel().repository,
            storage: context.readAppViewModel().storage,
            observer: observer,
          ),
          create: (context, args) => AuthViewModel(args),
          child: CartViewModelScope(
            args: (context) => CartViewModelArgs(observer: observer),
            create: (context, args) => CartViewModel(args),
            child: CheckoutViewModelScope(
              args: (context) => CheckoutViewModelArgs(
                repository: context.readAppViewModel().repository,
                observer: observer,
              ),
              create: (context, args) => CheckoutViewModel(args),
              child: OrdersViewModelScope(
                args: (context) => OrdersViewModelArgs(observer: observer),
                create: (context, args) => OrdersViewModel(args),
                child: OrderTrackingViewModelScope(
                  args: (context) => OrderTrackingViewModelArgs(
                    repository: context.readAppViewModel().repository,
                    observer: observer,
                  ),
                  create: (context, args) => OrderTrackingViewModel(args),
                  child: ProductsViewModelScope(
                    args: (context) => ProductsViewModelArgs(
                      repository: context.readAppViewModel().repository,
                      observer: observer,
                    ),
                    create: (context, args) => ProductsViewModel(args),
                    child: ProductDetailViewModelScope(
                      args: (context) => ProductDetailViewModelArgs(
                        repository: context.readAppViewModel().repository,
                        observer: observer,
                      ),
                      create: (context, args) => ProductDetailViewModel(args),
                      child: WishlistViewModelScope(
                        args: (context) => WishlistViewModelArgs(
                          storage: context.readAppViewModel().storage,
                          observer: observer,
                        ),
                        create: (context, args) => WishlistViewModel(args),
                        child: DemoCartApiViewModelScope(
                          args: (context) => DemoCartApiViewModelArgs(
                            repository: context.readAppViewModel().repository,
                            observer: observer,
                          ),
                          create: (context, args) => DemoCartApiViewModel(args),
                          child: ShoppingChatViewModelScope(
                            args: (context) => ShoppingChatViewModelArgs(
                              repository: context.readAppViewModel().repository,
                              observer: observer,
                            ),
                            create: (context, args) =>
                                ShoppingChatViewModel(args),
                            child: const _ShoppingRouterApp(),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _ShoppingRouterApp extends StatefulWidget {
  const _ShoppingRouterApp();

  @override
  State<_ShoppingRouterApp> createState() => _ShoppingRouterAppState();
}

class _ShoppingRouterAppState extends State<_ShoppingRouterApp> {
  AppRouter? _router;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _router ??= AppRouter(auth: AuthViewModelScope.read(context));
  }

  @override
  Widget build(BuildContext context) {
    final router = _router;
    if (router == null) {
      return const MaterialApp(
        home: Scaffold(body: Center(child: CircularProgressIndicator())),
      );
    }

    return MaterialApp.router(
      title: 'Dust Shopping App',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      routerConfig: router.config,
    );
  }
}
