import 'dart:async';
import 'package:flutter/material.dart';
import 'package:shopping_app/core/data/shopping_repository.dart';
import 'package:shopping_app/features/products/models/products_state.dart';
import 'package:shopping_app/features/products/view_models/products_view_model.dart';

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

class FailingInitProductsViewModel extends ProductsViewModel {
  FailingInitProductsViewModel(super.args);

  final initCompleters = <Completer<void>>[];
  var initCalls = 0;

  @override
  Future<void> onInit() {
    initCalls += 1;
    final completer = Completer<void>();
    initCompleters.add(completer);
    return completer.future;
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
