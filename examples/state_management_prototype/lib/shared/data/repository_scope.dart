import 'package:flutter/widgets.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/data/state_observer.dart';

class RepositoryScope extends InheritedWidget {
  const RepositoryScope({
    super.key,
    required this.repository,
    required this.observer,
    required super.child,
  });

  final PrototypeRepository repository;
  final StateObserver observer;

  static RepositoryScope of(BuildContext context) {
    final scope = context.dependOnInheritedWidgetOfExactType<RepositoryScope>();
    if (scope == null) {
      throw StateError('No RepositoryScope found in context.');
    }
    return scope;
  }

  @override
  bool updateShouldNotify(RepositoryScope oldWidget) =>
      repository != oldWidget.repository || observer != oldWidget.observer;
}

extension RepositoryBuildContext on BuildContext {
  PrototypeRepository get repository => RepositoryScope.of(this).repository;
  StateObserver get observer => RepositoryScope.of(this).observer;
}
