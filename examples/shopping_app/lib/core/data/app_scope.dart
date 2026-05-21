import 'package:flutter/widgets.dart';

import 'shopping_repository.dart';
import '../services/storage_service.dart';

class AppScope extends InheritedWidget {
  const AppScope({
    required this.repository,
    required this.storage,
    required super.child,
    super.key,
  });

  final ShoppingRepository repository;
  final StorageService storage;

  static AppScope of(BuildContext context) {
    final scope = context.dependOnInheritedWidgetOfExactType<AppScope>();
    if (scope == null) {
      throw StateError('No AppScope found in context.');
    }
    return scope;
  }

  @override
  bool updateShouldNotify(AppScope oldWidget) =>
      repository != oldWidget.repository || storage != oldWidget.storage;
}

extension AppScopeContext on BuildContext {
  ShoppingRepository get shoppingRepository => AppScope.of(this).repository;
  StorageService get storage => AppScope.of(this).storage;
}
