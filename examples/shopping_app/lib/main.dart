import 'dart:async' show unawaited;

import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';
import 'package:dust_flutter/state.dart';
import 'package:flutter_web_plugins/url_strategy.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'core/data/default_shopping_repository.dart';
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
import 'i18n/app_i18n.g.dart';
import 'route.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  usePathUrlStrategy();
  final prefs = await SharedPreferences.getInstance();
  runApp(
    AppI18n(
      child: ShoppingApp(storage: StorageService(prefs)),
    ),
  );
}

class ShoppingApp extends StatefulWidget {
  const ShoppingApp({
    required this.storage,
    this.repository,
    super.key,
  });

  final StorageService storage;
  final ShoppingRepository? repository;

  @override
  State<ShoppingApp> createState() => _ShoppingAppState();
}

class _ShoppingAppState extends State<ShoppingApp> {
  late final ShoppingRepository _repository =
      widget.repository ?? createDefaultShoppingRepository();

  @override
  void dispose() {
    unawaited(closeDefaultShoppingRepository(_repository));
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final observer = const LoggingStateObserver();
    return AppViewModelScope(
      args: (context) => AppViewModelArgs(
        repository: _repository,
        storage: widget.storage,
        observer: observer,
      ),
      create: (context, args) => AppViewModel(args),
      child: Builder(
        builder: (context) => AuthViewModelScope(
          args: (context) => AuthViewModelArgs(
            repository: context.readAppViewModel().args.repository,
            storage: context.readAppViewModel().args.storage,
            observer: observer,
          ),
          create: (context, args) => AuthViewModel(args),
          child: CartViewModelScope(
            args: (context) => CartViewModelArgs(observer: observer),
            create: (context, args) => CartViewModel(args),
            child: CheckoutViewModelScope(
              args: (context) => CheckoutViewModelArgs(
                repository: context.readAppViewModel().args.repository,
                observer: observer,
              ),
              create: (context, args) => CheckoutViewModel(args),
              child: OrdersViewModelScope(
                args: (context) => OrdersViewModelArgs(observer: observer),
                create: (context, args) => OrdersViewModel(args),
                child: OrderTrackingViewModelScope(
                  args: (context) => OrderTrackingViewModelArgs(
                    repository: context.readAppViewModel().args.repository,
                    observer: observer,
                  ),
                  create: (context, args) => OrderTrackingViewModel(args),
                  child: ProductsViewModelScope(
                    args: (context) => ProductsViewModelArgs(
                      repository: context.readAppViewModel().args.repository,
                      observer: observer,
                    ),
                    create: (context, args) => ProductsViewModel(args),
                    child: ProductDetailViewModelScope(
                      args: (context) => ProductDetailViewModelArgs(
                        repository: context.readAppViewModel().args.repository,
                        observer: observer,
                      ),
                      create: (context, args) => ProductDetailViewModel(args),
                      child: WishlistViewModelScope(
                        args: (context) => WishlistViewModelArgs(
                          storage: context.readAppViewModel().args.storage,
                          observer: observer,
                        ),
                        create: (context, args) => WishlistViewModel(args),
                        child: DemoCartApiViewModelScope(
                          args: (context) => DemoCartApiViewModelArgs(
                            repository:
                                context.readAppViewModel().args.repository,
                            observer: observer,
                          ),
                          create: (context, args) => DemoCartApiViewModel(args),
                          child: ShoppingChatViewModelScope(
                            args: (context) => ShoppingChatViewModelArgs(
                              repository:
                                  context.readAppViewModel().args.repository,
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
  ShoppingRouter? _router;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _router ??= ShoppingRouter(auth: AuthViewModelScope.read(context));
  }

  @override
  Widget build(BuildContext context) {
    final router = _router;
    final i18n = I18nScope.of(context);
    final locale = appI18nLocaleOf(i18n.locale);
    if (router == null) {
      return MaterialApp(
        locale: locale,
        supportedLocales: appI18nSupportedLocales,
        localizationsDelegates: appI18nLocalizationsDelegates,
        home: const Scaffold(body: Center(child: CircularProgressIndicator())),
      );
    }

    return MaterialApp.router(
      onGenerateTitle: (context) => i18n.translate(
        'shop_title',
        defaultText: 'Dust Shopping App',
      ),
      debugShowCheckedModeBanner: false,
      locale: locale,
      supportedLocales: appI18nSupportedLocales,
      localizationsDelegates: appI18nLocalizationsDelegates,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      routerConfig: router.config,
    );
  }
}
