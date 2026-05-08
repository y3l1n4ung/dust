import 'package:ease_state_helper/ease_state_helper.dart';
import 'package:ease_example/view_models/theme_view_model.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import 'middleware/time_machine_middleware.dart';
import 'router/app_router.dart';
import 'view_models/auth_view_model.dart';
import 'view_models/cart_view_model.dart';
import 'view_models/chat_view_model.dart';
import 'view_models/counter_view_model.dart';
import 'view_models/drawing_view_model.dart';
import 'view_models/form_view_model.dart';
import 'view_models/network_view_model.dart';
import 'view_models/pagination_view_model.dart';
import 'view_models/search_view_model.dart';
import 'view_models/side_effect_view_model.dart';
import 'view_models/todo_view_model.dart';

final provders = <ProviderBuilder>[
  (child) => AuthViewModelProvider(child: child),
  (child) => CartViewModelProvider(child: child),
  (child) => ChatViewModelProvider(child: child),
  (child) => CounterViewModelProvider(child: child),
  (child) => DrawingViewModelProvider(child: child),
  (child) => RegistrationFormViewModelProvider(child: child),
  (child) => NetworkViewModelProvider(child: child),
  (child) => PaginationViewModelProvider(child: child),
  (child) => SearchViewModelProvider(child: child),
  (child) => SideEffectViewModelProvider(child: child),
  (child) => ThemeViewModelProvider(child: child),
  (child) => TodoViewModelProvider(child: child),
];

void main() {
  initializeEaseDevTool(); // Initialize DevTools (debug mode only)
  StateNotifier.middleware = [
    LoggingMiddleware(),
    TimeMachineMiddleware(), // Undo/redo support
  ];
  runApp(EaseScope(providers: provders, child: const MyApp()));
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => _MyAppState();
}

class _MyAppState extends State<MyApp> {
  GoRouter? _router;
  bool _initialized = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (!_initialized) {
      _initialized = true;
      final authViewModel = context.readAuthViewModel();
      // Create router with auth viewmodel
      _router = createRouter(authViewModel);
      // Initialize auth from shared_preferences after build
      WidgetsBinding.instance.addPostFrameCallback((_) {
        authViewModel.initialize();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    // Show loading until router is initialized
    if (_router == null) {
      return const MaterialApp(
        home: Scaffold(body: Center(child: CircularProgressIndicator())),
      );
    }

    final themeViewModel = context.themeViewModel;

    return MaterialApp.router(
      title: 'Ease Examples',
      debugShowCheckedModeBanner: false,
      theme: themeViewModel.state.lightTheme,
      darkTheme: themeViewModel.state.darkTheme,
      themeMode: themeViewModel.state.themeMode,
      routerConfig: _router,
    );
  }
}
